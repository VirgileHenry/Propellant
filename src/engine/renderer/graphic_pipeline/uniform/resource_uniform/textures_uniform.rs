use std::collections::BTreeSet;

use crate::engine::consts::PROPELLANT_DEBUG_FEATURES;
use crate::engine::consts::PROPELLANT_MAX_LOADED_TEXTURE_COUNT;
use crate::engine::errors::PResult;
use crate::engine::resources::texture_library::TextureLibrary;
use crate::engine::window::vulkan::sync::VulkanSyncState;
use super::{
    ResourceUniform,
    ResourceUniformBuilder,
};

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

const START_DESCRIPTOR_SIZE: u32 = 16;

#[derive(Debug)]
pub struct TextureUniformBuilder {
    stage: vulkanalia::vk::ShaderStageFlags,
    binding: u32,
}

impl TextureUniformBuilder {
    pub fn new(binding: u32, stage: vulkanalia::vk::ShaderStageFlags) -> Self {
        Self {
            stage,
            binding,
        }
    }

    fn binding(&self) -> u32 {
        self.binding
    }

    fn stage(&self) -> vulkanalia::vk::ShaderStageFlags {
        self.stage
    }
}

impl ResourceUniformBuilder for TextureUniformBuilder {
    fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
    ) -> PResult<Box<dyn ResourceUniform>> {
        Ok(
            Box::new(
                TextureUniform::new(
                    self,
                    vk_device,
                    vk_descriptor_pool,
                )?
            )
        )
    }

    fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType {
        vulkanalia::vk::DescriptorType::COMBINED_IMAGE_SAMPLER
    }
}

#[derive(Debug)]
pub struct TextureUniform {
    /// DS layout
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// descriptor set. We only need a single one for the read only textures.
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
            .binding(builder.binding())
            .descriptor_type(builder.descriptor_type()) 
            .stage_flags(builder.stage())
            .descriptor_count(PROPELLANT_MAX_LOADED_TEXTURE_COUNT); // random value, will be modified after querying max count

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
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        textures: &TextureLibrary,
    ) -> PResult<()> {
        // populate the descriptor sets.
        // start by checking if we need to reallocate the descriptor set.
        if textures.max_index() >= self.descriptor_size {
            // need to reallocate the descriptor set.
            self.descriptor_size *= 2;

            if PROPELLANT_DEBUG_FEATURES {
                // check for max textures
                if self.descriptor_size > PROPELLANT_MAX_LOADED_TEXTURE_COUNT {
                    panic!("[PROPELLANT DEBUG] TextureUniform::populate_descriptor_sets: descriptor size is bigger than the max texture count.")
                }
            }

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

}

impl ResourceUniform for TextureUniform {
    fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }
    
    fn set(&self, _image_index: usize) -> vulkanalia::vk::DescriptorSet {
        match &self.descriptor_set {
            VulkanSyncState::Sane(set) => *set,
            VulkanSyncState::Syncing(sets) => {
                // todo : clean up older sets if no longer in use.
                sets[sets.len() - 1]
            },
        }
    }
    
    fn recreate(
        &mut self,
        vk_device: &vulkanalia::Device,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        resources: &crate::ProppellantResources,
    ) -> PResult<()> {
        // recreate our descriptor set + layout to match the built ressources.
        self.populate_descriptor_sets(
            vk_device,
            descriptor_pool,
            resources.textures(),
        )
    }
    
    fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}