use std::io::Cursor;

use wgpu::{
    Adapter, Device,
};

use futures::executor::block_on;

const COMP_SHADER_1: &[u8] = include_bytes!("../compiled-shaders/simple-comp.spv");

async fn run() -> Result<(), String> {
    let adapter = Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        },
        wgpu::BackendBit::PRIMARY,
    ).await.ok_or("No adapter available")?;

    eprintln!("Running on {:?}", adapter.get_info());

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await;

    // Make buffer

    let buffer_conts = (0..65536).map(|x| x as f32).collect::<Vec<_>>();

    let byte_buf = bytemuck::cast_slice::<f32, u8>(&buffer_conts);

    let buf = device.create_buffer_with_data(
        byte_buf,
        wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::STORAGE // what is a storage?
    );

    // Load compute shader

    let comp_shader_spirv = wgpu::read_spirv(Cursor::new(COMP_SHADER_1)).map_err(|e| e.to_string())?;
    let comp_mod = device.create_shader_module(&comp_shader_spirv);

    let bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                    },
                },
            ],
        },
    );

    let bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &buf,
                        range: 0..byte_buf.len() as u64,
                    },
                },
            ],
        },
    );

    let comp_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &bind_group_layout,
            ],
        },
    );

    let compute_pipeline = device.create_compute_pipeline(
        &wgpu::ComputePipelineDescriptor {
            layout: &comp_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &comp_mod,
                entry_point: "main",
            },
        }
    );

    // Dispatch!

    let mut encoder = device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor {
            label: None,
        }
    );

    {
        let mut pass = encoder.begin_compute_pass();
        pass.set_pipeline(&compute_pipeline);
        pass.set_bind_group(
            0,
            &bind_group,
            &[],
        );

        pass.dispatch(65536 / 64, 1, 1);
    }

    let cmd_buf = encoder.finish();

    println!("Submitting");
    queue.submit(&[cmd_buf]);
    println!("Done");

    // Read buffer

    let mapped_fut = buf.map_read(0, byte_buf.len() as u64);
    device.poll(wgpu::Maintain::Wait);
    let mapped_mem = mapped_fut.await.map_err(|_| "mapping failed")?;
    let conts = bytemuck::cast_slice::<u8, f32>(mapped_mem.as_slice());

    println!("Got back {:?}", &conts[..12]);

    Ok(())
}

fn main() {
    if let Err(e) = block_on(run()) {
        eprintln!("died: {:?}", e);
    }
}
