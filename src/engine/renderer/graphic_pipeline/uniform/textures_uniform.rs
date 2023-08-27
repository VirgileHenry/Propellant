use std::collections::BTreeSet;

use crate::engine::consts::PROPELLANT_MAX_LOADED_TEXTURE_COUNT;
use crate::engine::errors::PResult;
use crate::engine::resources::texture_library::TextureLibrary;
use crate::engine::window::vulkan::sync_state::VulkanSyncState;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

const START_DESCRIPTOR_SIZE: u32 = 16;

#[derive(Debug)]
pub struct TextureUniformBuilder {
    stage: vulkanalia::vk::ShaderStageFlags,
}

impl TextureUniformBuilder {
    pub fn new(stage: vulkanalia::vk::ShaderStageFlags) -> Self {
        Self {
            stage,
        }
    }

    fn stage(&self) -> vulkanalia::vk::ShaderStageFlags {
        self.stage
    }

    pub fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType {
        vulkanalia::vk::DescriptorType::COMBINED_IMAGE_SAMPLER
    }

    pub fn build(
        self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        _image_count: usize,
    ) -> PResult<TextureUniform> {
        TextureUniform::new(
            &self,
            vk_device,
            vk_descriptor_pool,
        )
    }
}


#[derive(Debug)]
pub struct TextureUniform {
    /// DS layout
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// descriptor set. We only need a single one for the read only textures.
    /// TODO : remove this sync, renderer already syncing us. Use a vec here instead, like everywhere else.
    descriptor_set: VulkanSyncState<vulkanalia::vk::DescriptorSet>,
    /// The current number of descriptor we can have without reallocating the descriptor set. 
    descriptor_size: u32,
    /// Keep track of loaded texture index.
    loaded_textures: BTreeSet<u32>,
}

impl TextureUniform {

    pub fn new(
        builder: &TextureUniformBuilder,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
    ) -> PResult<Self> {
        // create the descriptor set layout
        let binding_flags = [
            vulkanalia::vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT |
            vulkanalia::vk::DescriptorBindingFlags::PARTIALLY_BOUND |
            vulkanalia::vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
        ];
        let mut flags = vulkanalia::vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
            .binding_flags(&binding_flags);

        // the layout is a blueprint on how the descriptor set matches the shader.
        let layout_builder = vulkanalia::vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vulkanalia::vk::DescriptorType::COMBINED_IMAGE_SAMPLER) 
            .stage_flags(builder.stage())
            .descriptor_count(PROPELLANT_MAX_LOADED_TEXTURE_COUNT);

        let layout_bindings = [layout_builder];

        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings)
            .push_next(&mut flags);


        let layout = unsafe { vk_device.create_descriptor_set_layout(&info, None)? };

        let descriptor_set = Self::create_descriptor_set(
            vk_device,
            vk_descriptor_pool,
            layout,
            START_DESCRIPTOR_SIZE,
        )?;


        Ok(TextureUniform {
            layout,
            descriptor_set: VulkanSyncState::new(descriptor_set),
            descriptor_size: START_DESCRIPTOR_SIZE,
            loaded_textures: BTreeSet::new(),
        })
    }

    pub fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }

    /// Create our descriptor sets from the given pool.
    /// The pool might overflow, so in the future we should look into reallocating the pool.
    /// Creation would usually be done once at the start of the app.
    fn create_descriptor_set(
        vk_device: &vulkanalia::Device,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        layout: vulkanalia::vk::DescriptorSetLayout,
        descriptor_size: u32,
    ) -> PResult<vulkanalia::vk::DescriptorSet> {

        // create one descriptor set per swapchain image.
        let layouts = vec![layout];
        // dynamic descriptor count layout !
        let counts = vec![descriptor_size];
        let mut dynamic_descriptor_count = vulkanalia::vk::DescriptorSetVariableDescriptorCountAllocateInfo::builder()
            .descriptor_counts(&counts);

        let info = vulkanalia::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts)
            .push_next(&mut dynamic_descriptor_count);

        let sets = unsafe { vk_device.allocate_descriptor_sets(&info)? };

        Ok(sets[0])
    }

    /// Populate the descriptor sets with the resource lib
    pub fn populate_descriptor_sets(
        &mut self,
        vk_device: &vulkanalia::Device,
        _image_index: usize,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        textures: &TextureLibrary,
    ) -> PResult<()> {
        // populate the descriptor sets.
        // start by checking if we need to reallocate the descriptor set.
        if textures.max_index() > self.descriptor_size {
            // need to reallocate the descriptor set.
            self.descriptor_size *= 2;
            let new_ds = Self::create_descriptor_set(
                vk_device,
                descriptor_pool,
                self.layout,
                self.descriptor_size,
            )?;

            // clear the loaded textures, as we need to relaod all of them
            self.loaded_textures.clear();

            // update our state to syncing.
            match &mut self.descriptor_set {
                VulkanSyncState::Sane(set) => self.descriptor_set = VulkanSyncState::Syncing(vec![*set, new_ds].into()),
                VulkanSyncState::Syncing(sets) => sets.push_back(new_ds),
            }
        }

        let infos = textures.textures()
            .filter(|(index, _texture)| {
                self.loaded_textures.insert(*index) // will return true if the value was inserted, so the texture wasn't loaded.
            })
            .map(|(index, texture)| {
                ([
                    vulkanalia::vk::DescriptorImageInfo::builder()
                        .image_layout(vulkanalia::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(texture.view())
                        .sampler(texture.sampler())
                ], index)
            }).collect::<Vec<_>>();

        let sampler_writes = infos.iter().map(|(image_info, index)| {
            vulkanalia::vk::WriteDescriptorSet::builder()
                .dst_set(match &self.descriptor_set {
                    // write in the new set if syncing
                    VulkanSyncState::Sane(set) => *set,
                    VulkanSyncState::Syncing(sets) => sets[sets.len() - 1], 
                })
                .dst_binding(0)
                .dst_array_element(*index)
                .descriptor_type(vulkanalia::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_info.as_ref())
        }).collect::<Vec<_>>();

        unsafe { 
            vk_device.update_descriptor_sets(&sampler_writes, &[] as &[vulkanalia::vk::CopyDescriptorSet]);
        }

        Ok(())
    }

    pub fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        match &self.descriptor_set {
            VulkanSyncState::Sane(set) => *set,
            VulkanSyncState::Syncing(sets) => sets[image_index],
        }
    }

    // How hacky is this ? the textures uniform don't have a buffer, so these calls will be opt out.
    // these funcs allows us to be treated as a frame uniform in the pipeline gen macro.

    pub fn destroy_buffer(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
        }
    }

}
