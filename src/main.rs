#![no_std]
#![no_main]
extern crate alloc;

mod icosphere;
use icosphere::{icosphere, Vertex};

use core::ptr;
use psp::Align16;
use psp::sys::{
    self, ScePspFVector3, DisplayPixelFormat, GuContextType, GuSyncMode, GuSyncBehavior,
    GuPrimitive, FrontFaceDirection, ShadingModel, GuState, TexturePixelFormat, DepthFunc,
    VertexType, ClearBuffer, LightType, LightComponent,
};
use psp::vram_alloc::get_vram_allocator;
use psp::{BUF_WIDTH, SCREEN_WIDTH, SCREEN_HEIGHT};

psp::module!("sample_module", 1, 1);

static mut OPS_LIST: Align16<[u32; 0x40000]> = Align16([0; 0x40000]);

enum PSPError {
    InitError
}

// Angle that aligns the pole axis (0,A,B) to +Z: cos = B, sin = A → α = arcsin(A)
const POLE_ALIGN: f32 = 0.5535744;

unsafe fn init() -> Result<(), PSPError> {
    let allocator = get_vram_allocator().unwrap();
    let fbp0 = allocator.alloc_texture_pixels(BUF_WIDTH, SCREEN_HEIGHT, TexturePixelFormat::Psm8888);
    let fbp1 = allocator.alloc_texture_pixels(BUF_WIDTH, SCREEN_HEIGHT, TexturePixelFormat::Psm8888);
    let zbp = allocator.alloc_texture_pixels(BUF_WIDTH, SCREEN_HEIGHT, TexturePixelFormat::Psm4444);
    // Attempting to free the three VRAM chunks at this point would give a
    // compile-time error since fbp0, fbp1 and zbp are used later on
    //allocator.free_all();

    sys::sceGumLoadIdentity();

    sys::sceGuInit();

    sys::sceGuStart(GuContextType::Direct, &raw mut OPS_LIST.0 as *mut [u32; 0x40000] as *mut _);
    sys::sceGuDrawBuffer(DisplayPixelFormat::Psm8888, fbp0.as_mut_ptr_from_zero() as _, BUF_WIDTH as i32);
    sys::sceGuDispBuffer(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32, fbp1.as_mut_ptr_from_zero() as _, BUF_WIDTH as i32);
    sys::sceGuDepthBuffer(zbp.as_mut_ptr_from_zero() as _, BUF_WIDTH as i32);
    sys::sceGuOffset(2048 - (SCREEN_WIDTH / 2), 2048 - (SCREEN_HEIGHT / 2));
    sys::sceGuViewport(2048, 2048, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
    sys::sceGuDepthRange(65535, 0);
    sys::sceGuScissor(0, 0, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
    sys::sceGuEnable(GuState::ScissorTest);
    sys::sceGuDepthFunc(DepthFunc::GreaterOrEqual);
    sys::sceGuEnable(GuState::DepthTest);
    sys::sceGuFrontFace(FrontFaceDirection::Clockwise);
    sys::sceGuShadeModel(ShadingModel::Smooth);
    sys::sceGuEnable(GuState::CullFace);
    sys::sceGuEnable(GuState::ClipPlanes);

    // Lighting
    sys::sceGuEnable(GuState::Lighting);
    sys::sceGuEnable(GuState::Light0);
    let dir = ScePspFVector3 { x: 0.577, y: 0.577, z: 0.577 };
    sys::sceGuLight(0, LightType::Directional, LightComponent::DIFFUSE, &dir);
    sys::sceGuLightColor(0, LightComponent::DIFFUSE, 0xffffffff);
    sys::sceGuAmbient(0xff202020);

    sys::sceGuFinish();
    sys::sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);
    Ok(())
}

unsafe fn main_loop() {
    psp::sys::sceDisplayWaitVblankStart();

    sys::sceGuDisplay(true);

    let verts = icosphere(3);
    unsafe {
        sys::sceKernelDcacheWritebackRange(
            verts.as_ptr() as *const _,
            (verts.len() * core::mem::size_of::<Vertex>()) as u32,
        );
    }

    let mut val = 0.0_f32;


    loop {
        sys::sceGuStart(GuContextType::Direct, &raw mut OPS_LIST.0 as *mut [u32; 0x40000] as *mut _);

        sys::sceGuClearColor(0xff000000);
        sys::sceGuClearDepth(0);
        sys::sceGuClear(ClearBuffer::COLOR_BUFFER_BIT | ClearBuffer::DEPTH_BUFFER_BIT);

        sys::sceGumMatrixMode(sys::MatrixMode::Projection);
        sys::sceGumLoadIdentity();
        sys::sceGumPerspective(75.0, 16.0 / 9.0, 0.5, 1000.0);

        sys::sceGumMatrixMode(sys::MatrixMode::View);
        sys::sceGumLoadIdentity();

        sys::sceGumMatrixMode(sys::MatrixMode::Model);
        sys::sceGumLoadIdentity();
        sys::sceGumTranslate(&ScePspFVector3 { x: 0.0, y: 0.0, z: -2.5 });
        sys::sceGumScale(&ScePspFVector3 { x: 1.5, y: 1.5, z: 1.5 });
        // Rotate about the pole axis (0,A,B) via change of basis:
        // R = Rx(-α) * Rz(val) * Rx(α), where Rx(α) maps the pole to +Z
        sys::sceGumRotateX(-POLE_ALIGN);
        sys::sceGumRotateZ(val);
        sys::sceGumRotateX(POLE_ALIGN);

        sys::sceGuMaterial(LightComponent::DIFFUSE, 0xff0000ff);

        sys::sceGumDrawArray(
            GuPrimitive::Triangles,
            VertexType::NORMAL_32BITF | VertexType::VERTEX_32BITF | VertexType::TRANSFORM_3D,
            verts.len() as i32,
            ptr::null_mut(),
            verts.as_ptr() as *const _,
        );

        sys::sceGuFinish();
        sys::sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);

        sys::sceDisplayWaitVblankStart();
        sys::sceGuSwapBuffers();

        val += 0.02;
    }
}

fn psp_main() {
    psp::enable_home_button();
    // psp::dprint!("Hello PSP from rust!");
    unsafe {
        init();
        main_loop();
    }
}
