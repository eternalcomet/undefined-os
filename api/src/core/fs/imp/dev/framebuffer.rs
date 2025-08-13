// Screen information are queried by applications using the FBIOGET_FSCREENINFO and FBIOGET_VSCREENINFO ioctls.
// Those ioctls take a pointer to a fb_fix_screeninfo and fb_var_screeninfo structure respectively.
// See: https://www.kernel.org/doc/html/latest/fb/api.html#screen-information

use crate::core::fs::pseudo::file::{DeviceMem, DeviceOps};
use crate::ptr::UserOutPtr;
use axdisplay::framebuffer_flush;
use axdriver_display::DisplayInfo;
use axerrno::LinuxError;
use axhal::mem::virt_to_phys;
use memory_addr::VirtAddr;
use undefined_vfs::{VfsError, VfsResult};

/// struct fb_fix_screeninfo stores device independent unchangeable information about the frame buffer device and the current format.
/// Those information can’t be directly modified by applications, but can be changed by the driver when an application modifies the format.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FbFixScreenInfo {
    pub id: [u8; 16],       // Identification string, e.g., "TT Builtin"
    pub smem_start: u64,    // Start of framebuffer memory (physical address)
    pub smem_len: u32,      // Length of framebuffer memory
    pub type_: u32,         // See FB_TYPE_*
    pub type_aux: u32,      // Interleave for interleaved planes
    pub visual: u32,        // See FB_VISUAL_*
    pub xpanstep: u16,      // Zero if no hardware panning
    pub ypanstep: u16,      // Zero if no hardware panning
    pub ywrapstep: u16,     // Zero if no hardware ywrap
    pub line_length: u32,   // Length of a line in bytes
    pub mmio_start: u64,    // Start of Memory Mapped I/O (physical address)
    pub mmio_len: u32,      // Length of Memory Mapped I/O
    pub accel: u32,         // Indicate to driver which specific chip/card we have
    pub capabilities: u16,  // See FB_CAP_*
    pub reserved: [u16; 2], // Reserved for future compatibility
}

/// struct fb_var_screeninfo stores device independent changeable information about a frame buffer device,
/// its current format and video mode, as well as other miscellaneous parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FbVarScreenInfo {
    pub xres: u32, // Visible resolution
    pub yres: u32,
    pub xres_virtual: u32, // Virtual resolution
    pub yres_virtual: u32,
    pub xoffset: u32, // Offset from virtual to visible
    pub yoffset: u32,
    pub bits_per_pixel: u32,        // Guess what
    pub grayscale: u32,             // 0 = color, 1 = grayscale, >1 = FOURCC
    pub red: FrameBufferBitfield,   // Bitfield in framebuffer memory if true color
    pub green: FrameBufferBitfield, // Else only length is significant
    pub blue: FrameBufferBitfield,
    pub transp: FrameBufferBitfield, // Transparency
    pub nonstd: u32,                 // Non-standard pixel format
    pub activate: u32,               // See FB_ACTIVATE_*
    pub height: u32,                 // Height of picture in mm
    pub width: u32,                  // Width of picture in mm
    pub accel_flags: u32,            // (OBSOLETE) see fb_info.flags
    pub pixclock: u32,               // Pixel clock in ps (pico seconds)
    pub left_margin: u32,            // Time from sync to picture
    pub right_margin: u32,           // Time from picture to sync
    pub upper_margin: u32,           // Time from sync to picture
    pub lower_margin: u32,
    pub hsync_len: u32,     // Length of horizontal sync
    pub vsync_len: u32,     // Length of vertical sync
    pub sync: u32,          // See FB_SYNC_*
    pub vmode: u32,         // See FB_VMODE_*
    pub rotate: u32,        // Angle we rotate counter-clockwise
    pub colorspace: u32,    // Colorspace for FOURCC-based modes
    pub reserved: [u32; 4], // Reserved for future compatibility
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameBufferBitfield {
    /// The beginning of bitfield.
    offset: u32,
    /// The length of bitfield.
    length: u32,
    /// Most significant bit is right(!= 0).
    msb_right: u32,
}

pub struct FrameBuffer {
    info: DisplayInfo,
}

impl FrameBuffer {
    pub fn new(info: DisplayInfo) -> Self {
        Self { info }
    }

    #[deny(clippy::mut_from_ref)]
    fn get_buffer(&self) -> &mut [u8] {
        let info = &self.info;
        unsafe { core::slice::from_raw_parts_mut(info.fb_base_vaddr as *mut u8, info.fb_size) }
    }
}

impl DeviceOps for FrameBuffer {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        let buffer = self.get_buffer();
        let offset = offset as usize;
        if offset >= buffer.len() {
            return Ok(0);
        }
        let read_len = buf.len().min(buffer.len() - offset);
        buf[..read_len].copy_from_slice(&buffer[offset..offset + read_len]);
        Ok(read_len)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        let buffer = self.get_buffer();
        let offset = offset as usize;
        if offset >= buffer.len() {
            return Err(VfsError::ENOSPC);
        }
        let write_len = buf.len().min(buffer.len() - offset);
        buffer[offset..offset + write_len].copy_from_slice(&buf[..write_len]);
        if write_len > 0 {
            // Flush the display to apply changes
            framebuffer_flush();
        }
        Ok(write_len)
    }

    fn get_device_mem(&self) -> Option<DeviceMem> {
        let physical_addr = virt_to_phys(VirtAddr::from(self.info.fb_base_vaddr));
        Some(DeviceMem {
            physical_addr: physical_addr.as_usize(),
            length: self.info.fb_size,
        })
    }

    fn ioctl(&self, op: u32, arg: usize) -> VfsResult<isize> {
        const FBIOGET_VSCREENINFO: u32 = 0x4600;
        const FBIOGET_FSCREENINFO: u32 = 0x4602;
        const FBIO_WAITFORVSYNC: u32 = 0x4004_4620;
        match op {
            FBIOGET_VSCREENINFO => {
                let info = FbVarScreenInfo {
                    xres: self.info.width,
                    yres: self.info.height,
                    xres_virtual: self.info.width,
                    yres_virtual: self.info.height,
                    // RGBA8888
                    bits_per_pixel: 8 * 4,
                    red: FrameBufferBitfield {
                        offset: 0,
                        length: 8,
                        msb_right: 0,
                    },
                    green: FrameBufferBitfield {
                        offset: 8,
                        length: 8,
                        msb_right: 0,
                    },
                    blue: FrameBufferBitfield {
                        offset: 16,
                        length: 8,
                        msb_right: 0,
                    },
                    transp: FrameBufferBitfield {
                        offset: 24,
                        length: 8,
                        msb_right: 0,
                    },
                    ..Default::default()
                };
                let user_info: UserOutPtr<FbVarScreenInfo> = arg.into();
                *user_info.get_as_mut_ref()? = info;
                Ok(0)
            }
            FBIOGET_FSCREENINFO => {
                let info = FbFixScreenInfo {
                    smem_start: self.info.fb_base_vaddr as _,
                    smem_len: self.info.fb_size as _,
                    line_length: self.info.width * 4, // Assuming 4 bytes per pixel (RGBA8888)
                    ..Default::default()
                };
                let user_info: UserOutPtr<FbFixScreenInfo> = arg.into();
                *user_info.get_as_mut_ref()? = info;
                Ok(0)
            }
            FBIO_WAITFORVSYNC => {
                // This ioctl is used to wait for the next vertical sync.
                // In a real implementation, this would block until the next vsync.
                // Here we just return 0 to indicate success.
                framebuffer_flush();
                Ok(0)
            }
            _ => Err(LinuxError::ENOTTY),
        }
    }
}
