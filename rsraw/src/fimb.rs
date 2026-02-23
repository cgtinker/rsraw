use std::borrow::Cow;
use std::ffi::CStr;

use rsraw_sys as sys;
use chrono::{DateTime, Local, TimeZone};

use crate::{
    err::{Error, Result},
    processed::ProcessedImage,
};

pub type BitDepth = u32;

pub const BIT_DEPTH_8: BitDepth = 8;
pub const BIT_DEPTH_16: BitDepth = 16;

pub struct RawImage {
    raw_data: *mut sys::libraw_data_t,
}

// ============================================================================
// Top-level accessors - return wrapper structs only
// ============================================================================
impl RawImage {
    fn as_ref(&self) -> &sys::libraw_data_t {
        unsafe { &*self.raw_data }
    }

    fn as_mut_ref(&mut self) -> &mut sys::libraw_data_t {
        unsafe { &mut *self.raw_data }
    }

    pub fn open(buf: &[u8]) -> Result<Self> {
        let raw_data = unsafe { sys::libraw_init(0) };
        Error::check(unsafe {
            sys::libraw_open_buffer(raw_data, buf.as_ptr() as *const _, buf.len())
        })?;
        Ok(Self { raw_data })
    }

    pub fn unpack(&mut self) -> Result<()> {
        unsafe {
            let mut raw_params = self.rawparams();
            raw_params.set_use_rawspeed(true);
            raw_params.set_max_raw_memory_mb(1024);
            Error::check(sys::libraw_unpack(self.raw_data))
        }
    }

    pub fn process<const D: BitDepth>(&mut self) -> Result<ProcessedImage<D>> {
        debug_assert!(D == BIT_DEPTH_8 || D == BIT_DEPTH_16);
        unsafe { (*self.raw_data).params.output_bps = D as i32 };
        Error::check(unsafe { sys::libraw_dcraw_process(self.raw_data) })?;

        let mut result = 0i32;
        let processed = unsafe { sys::libraw_dcraw_make_mem_image(self.raw_data, &mut result) };
        Error::check(result)?;
        Ok(unsafe { ProcessedImage::from_raw(processed) })
    }

    pub fn processed_image(&self) -> Option<&[[u16; 4]]> {
        let ptr = self.as_ref().image;
        if ptr.is_null() {
            None
        } else {
            let w = self.as_ref().sizes.width as usize;
            let h = self.as_ref().sizes.height as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h) })
        }
    }

    pub fn idata(&self) -> ImageParams<'_> {
        ImageParams {
            data: &self.as_ref().idata,
        }
    }

    pub fn sizes(&self) -> ImageSizes<'_> {
        ImageSizes {
            data: &self.as_ref().sizes,
        }
    }

    pub fn lens(&self) -> LensInfo<'_> {
        LensInfo {
            data: &self.as_ref().lens,
        }
    }

    pub fn makernotes(&self) -> Makernotes<'_> {
        Makernotes {
            data: &self.as_ref().makernotes,
        }
    }

    pub fn shootinginfo(&self) -> ShootingInfo<'_> {
        ShootingInfo {
            data: &self.as_ref().shootinginfo,
        }
    }

    pub fn params(&self) -> OutputParams<'_> {
        OutputParams {
            data: &self.as_ref().params,
        }
    }

    pub fn params_mut(&mut self) -> OutputParamsMut<'_> {
        OutputParamsMut {
            data: &mut self.as_mut_ref().params,
        }
    }

    pub fn rawparams(&mut self) -> RawUnpackParams<'_> {
        RawUnpackParams {
            data: &mut self.as_mut_ref().rawparams,
        }
    }

    pub fn progress_flags(&self) -> u32 {
        self.as_ref().progress_flags
    }

    pub fn process_warnings(&self) -> u32 {
        self.as_ref().process_warnings
    }

    pub fn color(&self) -> ColorData<'_> {
        ColorData {
            data: &self.as_ref().color,
        }
    }

    pub fn other(&self) -> ImgOther<'_> {
        ImgOther {
            data: &self.as_ref().other,
        }
    }

    pub fn thumbnail(&self) -> Thumbnail<'_> {
        Thumbnail {
            data: &self.as_ref().thumbnail,
        }
    }

    pub fn thumbs_list(&self) -> ThumbsList<'_> {
        ThumbsList {
            data: &self.as_ref().thumbs_list,
        }
    }

    pub fn rawdata(&self) -> RawData<'_> {
        RawData {
            data: &self.as_ref().rawdata,
        }
    }
}

impl Drop for RawImage {
    fn drop(&mut self) {
        unsafe { sys::libraw_close(self.raw_data) }
    }
}

impl AsRef<sys::libraw_data_t> for RawImage {
    fn as_ref(&self) -> &sys::libraw_data_t {
        unsafe { &*self.raw_data }
    }
}

impl AsMut<sys::libraw_data_t> for RawImage {
    fn as_mut(&mut self) -> &mut sys::libraw_data_t {
        unsafe { &mut *self.raw_data }
    }
}

// ============================================================================
// Image Parameters
// ============================================================================
pub struct ImageParams<'a> {
    data: &'a sys::libraw_iparams_t,
}

impl<'a> ImageParams<'a> {
    pub fn guard(&self) -> &[u8; 4] {
        unsafe { std::mem::transmute(&self.data.guard) }
    }

    pub fn make(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.make[0]).to_string_lossy() }
    }

    pub fn model(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.model[0]).to_string_lossy() }
    }

    pub fn software(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.software[0]).to_string_lossy() }
    }

    pub fn normalized_make(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.normalized_make[0]).to_string_lossy() }
    }

    pub fn normalized_model(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.normalized_model[0]).to_string_lossy() }
    }

    pub fn maker_index(&self) -> u32 {
        self.data.maker_index
    }

    pub fn raw_count(&self) -> u32 {
        self.data.raw_count
    }

    pub fn dng_version(&self) -> u32 {
        self.data.dng_version
    }

    pub fn is_foveon(&self) -> bool {
        self.data.is_foveon != 0
    }

    pub fn colors(&self) -> i32 {
        self.data.colors
    }

    pub fn filters(&self) -> u32 {
        self.data.filters
    }

    pub fn xtrans(&self) -> &[[i8; 6]; 6] {
        &self.data.xtrans
    }

    pub fn xtrans_abs(&self) -> &[[i8; 6]; 6] {
        &self.data.xtrans_abs
    }

    pub fn cdesc(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.cdesc[0]).to_string_lossy() }
    }

    pub fn xmp_data(&self) -> Option<&[u8]> {
        if self.data.xmpdata.is_null() || self.data.xmplen == 0 {
            None
        } else {
            Some(unsafe {
                std::slice::from_raw_parts(
                    self.data.xmpdata as *const u8,
                    self.data.xmplen as usize,
                )
            })
        }
    }
}

// ============================================================================
// Image Sizes
// ============================================================================
pub struct ImageSizes<'a> {
    data: &'a sys::libraw_image_sizes_t,
}

impl<'a> ImageSizes<'a> {
    pub fn raw_height(&self) -> u16 {
        self.data.raw_height
    }

    pub fn raw_width(&self) -> u16 {
        self.data.raw_width
    }

    pub fn height(&self) -> u16 {
        self.data.height
    }

    pub fn width(&self) -> u16 {
        self.data.width
    }

    pub fn top_margin(&self) -> u16 {
        self.data.top_margin
    }

    pub fn left_margin(&self) -> u16 {
        self.data.left_margin
    }

    pub fn iheight(&self) -> u16 {
        self.data.iheight
    }

    pub fn iwidth(&self) -> u16 {
        self.data.iwidth
    }

    pub fn raw_pitch(&self) -> u32 {
        self.data.raw_pitch
    }

    pub fn pixel_aspect(&self) -> f64 {
        self.data.pixel_aspect
    }

    pub fn flip(&self) -> i32 {
        self.data.flip
    }

    pub fn mask(&self) -> &[[i32; 4]; 8] {
        &self.data.mask
    }

    pub fn raw_aspect(&self) -> u16 {
        self.data.raw_aspect
    }

    pub fn raw_inset_crops(&self) -> &[RawInsetCrop; 2] {
        unsafe { std::mem::transmute(&self.data.raw_inset_crops) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RawInsetCrop {
    pub cleft: u16,
    pub ctop: u16,
    pub cwidth: u16,
    pub cheight: u16,
}

// ============================================================================
// Lens Info
// ============================================================================
pub struct LensInfo<'a> {
    data: &'a sys::libraw_lensinfo_t,
}

impl<'a> LensInfo<'a> {
    pub fn min_focal(&self) -> f32 {
        self.data.MinFocal
    }

    pub fn max_focal(&self) -> f32 {
        self.data.MaxFocal
    }

    pub fn max_ap4_min_focal(&self) -> f32 {
        self.data.MaxAp4MinFocal
    }

    pub fn max_ap4_max_focal(&self) -> f32 {
        self.data.MaxAp4MaxFocal
    }

    pub fn exif_max_ap(&self) -> f32 {
        self.data.EXIF_MaxAp
    }

    pub fn lens_make(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.LensMake[0]).to_string_lossy() }
    }

    pub fn lens(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Lens[0]).to_string_lossy() }
    }

    pub fn lens_serial(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.LensSerial[0]).to_string_lossy() }
    }

    pub fn internal_lens_serial(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.InternalLensSerial[0]).to_string_lossy() }
    }

    pub fn focal_length_35mm(&self) -> u16 {
        self.data.FocalLengthIn35mmFormat
    }

    pub fn nikon(&self) -> NikonLens<'_> {
        NikonLens {
            data: &self.data.nikon,
        }
    }

    pub fn dng(&self) -> DngLens<'_> {
        DngLens {
            data: &self.data.dng,
        }
    }

    pub fn makernotes(&self) -> MakernotesLens<'_> {
        MakernotesLens {
            data: &self.data.makernotes,
        }
    }
}

pub struct NikonLens<'a> {
    data: &'a sys::libraw_nikonlens_t,
}

impl<'a> NikonLens<'a> {
    pub fn effective_max_ap(&self) -> f32 {
        self.data.EffectiveMaxAp
    }

    pub fn lens_id_number(&self) -> u8 {
        self.data.LensIDNumber
    }

    pub fn lens_f_stops(&self) -> u8 {
        self.data.LensFStops
    }

    pub fn mcu_version(&self) -> u8 {
        self.data.MCUVersion
    }

    pub fn lens_type(&self) -> u8 {
        self.data.LensType
    }
}

pub struct DngLens<'a> {
    data: &'a sys::libraw_dnglens_t,
}

impl<'a> DngLens<'a> {
    pub fn min_focal(&self) -> f32 {
        self.data.MinFocal
    }

    pub fn max_focal(&self) -> f32 {
        self.data.MaxFocal
    }

    pub fn max_ap4_min_focal(&self) -> f32 {
        self.data.MaxAp4MinFocal
    }

    pub fn max_ap4_max_focal(&self) -> f32 {
        self.data.MaxAp4MaxFocal
    }
}

pub struct MakernotesLens<'a> {
    data: &'a sys::libraw_makernotes_lens_t,
}

impl<'a> MakernotesLens<'a> {
    pub fn lens_id(&self) -> u64 {
        self.data.LensID
    }

    pub fn lens(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Lens[0]).to_string_lossy() }
    }

    pub fn lens_format(&self) -> u16 {
        self.data.LensFormat
    }

    pub fn lens_mount(&self) -> u16 {
        self.data.LensMount
    }

    pub fn cam_id(&self) -> u64 {
        self.data.CamID
    }

    pub fn camera_format(&self) -> u16 {
        self.data.CameraFormat
    }

    pub fn camera_mount(&self) -> u16 {
        self.data.CameraMount
    }

    pub fn body(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.body[0]).to_string_lossy() }
    }

    pub fn focal_type(&self) -> i16 {
        self.data.FocalType
    }

    pub fn lens_features_pre(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.LensFeatures_pre[0]).to_string_lossy() }
    }

    pub fn lens_features_suf(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.LensFeatures_suf[0]).to_string_lossy() }
    }

    pub fn min_focal(&self) -> f32 {
        self.data.MinFocal
    }

    pub fn max_focal(&self) -> f32 {
        self.data.MaxFocal
    }

    pub fn max_ap4_min_focal(&self) -> f32 {
        self.data.MaxAp4MinFocal
    }

    pub fn max_ap4_max_focal(&self) -> f32 {
        self.data.MaxAp4MaxFocal
    }

    pub fn min_ap4_min_focal(&self) -> f32 {
        self.data.MinAp4MinFocal
    }

    pub fn min_ap4_max_focal(&self) -> f32 {
        self.data.MinAp4MaxFocal
    }

    pub fn max_ap(&self) -> f32 {
        self.data.MaxAp
    }

    pub fn min_ap(&self) -> f32 {
        self.data.MinAp
    }

    pub fn cur_focal(&self) -> f32 {
        self.data.CurFocal
    }

    pub fn cur_ap(&self) -> f32 {
        self.data.CurAp
    }

    pub fn max_ap4_cur_focal(&self) -> f32 {
        self.data.MaxAp4CurFocal
    }

    pub fn min_ap4_cur_focal(&self) -> f32 {
        self.data.MinAp4CurFocal
    }

    pub fn min_focus_distance(&self) -> f32 {
        self.data.MinFocusDistance
    }

    pub fn focus_range_index(&self) -> f32 {
        self.data.FocusRangeIndex
    }

    pub fn lens_f_stops(&self) -> f32 {
        self.data.LensFStops
    }

    pub fn teleconverter_id(&self) -> u64 {
        self.data.TeleconverterID
    }

    pub fn teleconverter(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Teleconverter[0]).to_string_lossy() }
    }

    pub fn adapter_id(&self) -> u64 {
        self.data.AdapterID
    }

    pub fn adapter(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Adapter[0]).to_string_lossy() }
    }

    pub fn attachment_id(&self) -> u64 {
        self.data.AttachmentID
    }

    pub fn attachment(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Attachment[0]).to_string_lossy() }
    }

    pub fn focal_units(&self) -> u16 {
        self.data.FocalUnits
    }

    pub fn focal_length_35mm(&self) -> f32 {
        self.data.FocalLengthIn35mmFormat
    }
}

// ============================================================================
// Shooting Info
// ============================================================================
pub struct ShootingInfo<'a> {
    data: &'a sys::libraw_shootinginfo_t,
}

impl<'a> ShootingInfo<'a> {
    pub fn drive_mode(&self) -> i16 {
        self.data.DriveMode
    }

    pub fn focus_mode(&self) -> i16 {
        self.data.FocusMode
    }

    pub fn metering_mode(&self) -> i16 {
        self.data.MeteringMode
    }

    pub fn af_point(&self) -> i16 {
        self.data.AFPoint
    }

    pub fn exposure_mode(&self) -> i16 {
        self.data.ExposureMode
    }

    pub fn exposure_program(&self) -> i16 {
        self.data.ExposureProgram
    }

    pub fn image_stabilization(&self) -> i16 {
        self.data.ImageStabilization
    }

    pub fn body_serial(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.BodySerial[0]).to_string_lossy() }
    }

    pub fn internal_body_serial(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.InternalBodySerial[0]).to_string_lossy() }
    }
}

// ============================================================================
// Output Parameters
// ============================================================================
pub struct OutputParams<'a> {
    data: &'a sys::libraw_output_params_t,
}

impl<'a> OutputParams<'a> {
    pub fn greybox(&self) -> &[u32; 4] {
        &self.data.greybox
    }
    pub fn cropbox(&self) -> &[u32; 4] {
        &self.data.cropbox
    }
    pub fn aber(&self) -> &[f64; 4] {
        &self.data.aber
    }
    pub fn gamm(&self) -> &[f64; 6] {
        &self.data.gamm
    }
    pub fn user_mul(&self) -> &[f32; 4] {
        &self.data.user_mul
    }
    pub fn bright(&self) -> f32 {
        self.data.bright
    }
    pub fn threshold(&self) -> f32 {
        self.data.threshold
    }
    pub fn half_size(&self) -> bool {
        self.data.half_size != 0
    }
    pub fn four_color_rgb(&self) -> bool {
        self.data.four_color_rgb != 0
    }
    pub fn highlight(&self) -> i32 {
        self.data.highlight
    }
    pub fn use_auto_wb(&self) -> bool {
        self.data.use_auto_wb != 0
    }
    pub fn use_camera_wb(&self) -> bool {
        self.data.use_camera_wb != 0
    }
    pub fn use_camera_matrix(&self) -> i32 {
        self.data.use_camera_matrix
    }
    pub fn output_color(&self) -> i32 {
        self.data.output_color
    }
    pub fn output_profile(&self) -> Option<Cow<'_, str>> {
        if self.data.output_profile.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.output_profile).to_string_lossy() })
        }
    }
    pub fn camera_profile(&self) -> Option<Cow<'_, str>> {
        if self.data.camera_profile.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.camera_profile).to_string_lossy() })
        }
    }
    pub fn bad_pixels(&self) -> Option<Cow<'_, str>> {
        if self.data.bad_pixels.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.bad_pixels).to_string_lossy() })
        }
    }
    pub fn dark_frame(&self) -> Option<Cow<'_, str>> {
        if self.data.dark_frame.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.dark_frame).to_string_lossy() })
        }
    }
    pub fn output_bps(&self) -> i32 {
        self.data.output_bps
    }
    pub fn output_tiff(&self) -> bool {
        self.data.output_tiff != 0
    }
    pub fn output_flags(&self) -> i32 {
        self.data.output_flags
    }
    pub fn user_flip(&self) -> i32 {
        self.data.user_flip
    }
    pub fn user_qual(&self) -> i32 {
        self.data.user_qual
    }
    pub fn user_black(&self) -> i32 {
        self.data.user_black
    }
    pub fn user_cblack(&self) -> &[i32; 4] {
        &self.data.user_cblack
    }
    pub fn user_sat(&self) -> i32 {
        self.data.user_sat
    }
    pub fn med_passes(&self) -> i32 {
        self.data.med_passes
    }
    pub fn auto_bright_thr(&self) -> f32 {
        self.data.auto_bright_thr
    }
    pub fn adjust_maximum_thr(&self) -> f32 {
        self.data.adjust_maximum_thr
    }
    pub fn no_auto_bright(&self) -> bool {
        self.data.no_auto_bright != 0
    }
    pub fn use_fuji_rotate(&self) -> bool {
        self.data.use_fuji_rotate != 0
    }
    pub fn green_matching(&self) -> bool {
        self.data.green_matching != 0
    }
    pub fn dcb_iterations(&self) -> i32 {
        self.data.dcb_iterations
    }
    pub fn dcb_enhance_fl(&self) -> bool {
        self.data.dcb_enhance_fl != 0
    }
    pub fn fbdd_noiserd(&self) -> i32 {
        self.data.fbdd_noiserd
    }
    pub fn exp_correc(&self) -> bool {
        self.data.exp_correc != 0
    }
    pub fn exp_shift(&self) -> f32 {
        self.data.exp_shift
    }
    pub fn exp_preser(&self) -> f32 {
        self.data.exp_preser
    }
    pub fn no_auto_scale(&self) -> bool {
        self.data.no_auto_scale != 0
    }
    pub fn no_interpolation(&self) -> bool {
        self.data.no_interpolation != 0
    }
}

pub struct OutputParamsMut<'a> {
    data: &'a mut sys::libraw_output_params_t,
}

impl<'a> OutputParamsMut<'a> {
    pub fn greybox(&self) -> &[u32; 4] {
        &self.data.greybox
    }
    pub fn set_greybox(&mut self, v: [u32; 4]) {
        self.data.greybox = v;
    }
    pub fn cropbox(&self) -> &[u32; 4] {
        &self.data.cropbox
    }
    pub fn set_cropbox(&mut self, v: [u32; 4]) {
        self.data.cropbox = v;
    }
    pub fn aber(&self) -> &[f64; 4] {
        &self.data.aber
    }
    pub fn set_aber(&mut self, v: [f64; 4]) {
        self.data.aber = v;
    }
    pub fn gamm(&self) -> &[f64; 6] {
        &self.data.gamm
    }
    pub fn set_gamm(&mut self, v: [f64; 6]) {
        self.data.gamm = v;
    }
    pub fn user_mul(&self) -> &[f32; 4] {
        &self.data.user_mul
    }
    pub fn set_user_mul(&mut self, v: [f32; 4]) {
        self.data.user_mul = v;
    }
    pub fn bright(&self) -> f32 {
        self.data.bright
    }
    pub fn set_bright(&mut self, v: f32) {
        self.data.bright = v;
    }
    pub fn threshold(&self) -> f32 {
        self.data.threshold
    }
    pub fn set_threshold(&mut self, v: f32) {
        self.data.threshold = v;
    }
    pub fn half_size(&self) -> bool {
        self.data.half_size != 0
    }
    pub fn set_half_size(&mut self, v: bool) {
        self.data.half_size = v as i32;
    }
    pub fn four_color_rgb(&self) -> bool {
        self.data.four_color_rgb != 0
    }
    pub fn set_four_color_rgb(&mut self, v: bool) {
        self.data.four_color_rgb = v as i32;
    }
    pub fn highlight(&self) -> i32 {
        self.data.highlight
    }
    pub fn set_highlight(&mut self, v: i32) {
        self.data.highlight = v;
    }
    pub fn use_auto_wb(&self) -> bool {
        self.data.use_auto_wb != 0
    }
    pub fn set_use_auto_wb(&mut self, v: bool) {
        self.data.use_auto_wb = v as i32;
    }
    pub fn use_camera_wb(&self) -> bool {
        self.data.use_camera_wb != 0
    }
    pub fn set_use_camera_wb(&mut self, v: bool) {
        self.data.use_camera_wb = v as i32;
    }
    pub fn use_camera_matrix(&self) -> i32 {
        self.data.use_camera_matrix
    }
    pub fn set_use_camera_matrix(&mut self, v: i32) {
        self.data.use_camera_matrix = v;
    }
    pub fn output_color(&self) -> i32 {
        self.data.output_color
    }
    pub fn set_output_color(&mut self, v: i32) {
        self.data.output_color = v;
    }
    pub fn output_profile(&self) -> Option<Cow<'_, str>> {
        if self.data.output_profile.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.output_profile).to_string_lossy() })
        }
    }
    pub fn set_output_profile(&mut self, v: Option<&std::ffi::CStr>) {
        self.data.output_profile = v.map_or(std::ptr::null_mut(), |s| s.as_ptr() as *mut _);
    }
    pub fn camera_profile(&self) -> Option<Cow<'_, str>> {
        if self.data.camera_profile.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.camera_profile).to_string_lossy() })
        }
    }
    pub fn set_camera_profile(&mut self, v: Option<&std::ffi::CStr>) {
        self.data.camera_profile = v.map_or(std::ptr::null_mut(), |s| s.as_ptr() as *mut _);
    }
    pub fn bad_pixels(&self) -> Option<Cow<'_, str>> {
        if self.data.bad_pixels.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.bad_pixels).to_string_lossy() })
        }
    }
    pub fn set_bad_pixels(&mut self, v: Option<&std::ffi::CStr>) {
        self.data.bad_pixels = v.map_or(std::ptr::null_mut(), |s| s.as_ptr() as *mut _);
    }
    pub fn dark_frame(&self) -> Option<Cow<'_, str>> {
        if self.data.dark_frame.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.data.dark_frame).to_string_lossy() })
        }
    }
    pub fn set_dark_frame(&mut self, v: Option<&std::ffi::CStr>) {
        self.data.dark_frame = v.map_or(std::ptr::null_mut(), |s| s.as_ptr() as *mut _);
    }
    pub fn output_bps(&self) -> i32 {
        self.data.output_bps
    }
    pub fn set_output_bps(&mut self, v: i32) {
        self.data.output_bps = v;
    }
    pub fn output_tiff(&self) -> bool {
        self.data.output_tiff != 0
    }
    pub fn set_output_tiff(&mut self, v: bool) {
        self.data.output_tiff = v as i32;
    }
    pub fn output_flags(&self) -> i32 {
        self.data.output_flags
    }
    pub fn set_output_flags(&mut self, v: i32) {
        self.data.output_flags = v;
    }
    pub fn user_flip(&self) -> i32 {
        self.data.user_flip
    }
    pub fn set_user_flip(&mut self, v: i32) {
        self.data.user_flip = v;
    }
    pub fn user_qual(&self) -> i32 {
        self.data.user_qual
    }
    pub fn set_user_qual(&mut self, v: i32) {
        self.data.user_qual = v;
    }
    pub fn user_black(&self) -> i32 {
        self.data.user_black
    }
    pub fn set_user_black(&mut self, v: i32) {
        self.data.user_black = v;
    }
    pub fn user_cblack(&self) -> &[i32; 4] {
        &self.data.user_cblack
    }
    pub fn set_user_cblack(&mut self, v: [i32; 4]) {
        self.data.user_cblack = v;
    }
    pub fn user_sat(&self) -> i32 {
        self.data.user_sat
    }
    pub fn set_user_sat(&mut self, v: i32) {
        self.data.user_sat = v;
    }
    pub fn med_passes(&self) -> i32 {
        self.data.med_passes
    }
    pub fn set_med_passes(&mut self, v: i32) {
        self.data.med_passes = v;
    }
    pub fn auto_bright_thr(&self) -> f32 {
        self.data.auto_bright_thr
    }
    pub fn set_auto_bright_thr(&mut self, v: f32) {
        self.data.auto_bright_thr = v;
    }
    pub fn adjust_maximum_thr(&self) -> f32 {
        self.data.adjust_maximum_thr
    }
    pub fn set_adjust_maximum_thr(&mut self, v: f32) {
        self.data.adjust_maximum_thr = v;
    }
    pub fn no_auto_bright(&self) -> bool {
        self.data.no_auto_bright != 0
    }
    pub fn set_no_auto_bright(&mut self, v: bool) {
        self.data.no_auto_bright = v as i32;
    }
    pub fn use_fuji_rotate(&self) -> bool {
        self.data.use_fuji_rotate != 0
    }
    pub fn set_use_fuji_rotate(&mut self, v: bool) {
        self.data.use_fuji_rotate = v as i32;
    }
    pub fn green_matching(&self) -> bool {
        self.data.green_matching != 0
    }
    pub fn set_green_matching(&mut self, v: bool) {
        self.data.green_matching = v as i32;
    }
    pub fn dcb_iterations(&self) -> i32 {
        self.data.dcb_iterations
    }
    pub fn set_dcb_iterations(&mut self, v: i32) {
        self.data.dcb_iterations = v;
    }
    pub fn dcb_enhance_fl(&self) -> bool {
        self.data.dcb_enhance_fl != 0
    }
    pub fn set_dcb_enhance_fl(&mut self, v: bool) {
        self.data.dcb_enhance_fl = v as i32;
    }
    pub fn fbdd_noiserd(&self) -> i32 {
        self.data.fbdd_noiserd
    }
    pub fn set_fbdd_noiserd(&mut self, v: i32) {
        self.data.fbdd_noiserd = v;
    }
    pub fn exp_correc(&self) -> bool {
        self.data.exp_correc != 0
    }
    pub fn set_exp_correc(&mut self, v: bool) {
        self.data.exp_correc = v as i32;
    }
    pub fn exp_shift(&self) -> f32 {
        self.data.exp_shift
    }
    pub fn set_exp_shift(&mut self, v: f32) {
        self.data.exp_shift = v;
    }
    pub fn exp_preser(&self) -> f32 {
        self.data.exp_preser
    }
    pub fn set_exp_preser(&mut self, v: f32) {
        self.data.exp_preser = v;
    }
    pub fn no_auto_scale(&self) -> bool {
        self.data.no_auto_scale != 0
    }
    pub fn set_no_auto_scale(&mut self, v: bool) {
        self.data.no_auto_scale = v as i32;
    }
    pub fn no_interpolation(&self) -> bool {
        self.data.no_interpolation != 0
    }
    pub fn set_no_interpolation(&mut self, v: bool) {
        self.data.no_interpolation = v as i32;
    }
}

// ============================================================================
// Raw Unpack Parameters
// ============================================================================
pub struct RawUnpackParams<'a> {
    data: &'a mut sys::libraw_raw_unpack_params_t,
}

impl<'a> RawUnpackParams<'a> {
    pub fn set_use_rawspeed(&mut self, value: bool) {
        self.data.use_rawspeed = value as i32;
    }

    pub fn set_use_dngsdk(&mut self, value: bool) {
        self.data.use_dngsdk = value as i32;
    }

    pub fn set_options(&mut self, options: u32) {
        self.data.options = options;
    }

    pub fn set_shot_select(&mut self, shot: u32) {
        self.data.shot_select = shot;
    }

    pub fn set_max_raw_memory_mb(&mut self, mb: u32) {
        self.data.max_raw_memory_mb = mb;
    }

    pub fn set_coolscan_nef_gamma(&mut self, gamma: f32) {
        self.data.coolscan_nef_gamma = gamma;
    }

    pub fn set_p4shot_order(&mut self, s: &CStr) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                s.as_ptr(),
                self.data.p4shot_order.as_mut_ptr(),
                self.data.p4shot_order.len(),
            );
        }
    }

    pub fn use_rawspeed(&self) -> bool {
        self.data.use_rawspeed != 0
    }

    pub fn use_dngsdk(&self) -> bool {
        self.data.use_dngsdk != 0
    }

    pub fn options(&self) -> u32 {
        self.data.options
    }

    pub fn shot_select(&self) -> u32 {
        self.data.shot_select
    }

    pub fn specials(&self) -> u32 {
        self.data.specials
    }

    pub fn max_raw_memory_mb(&self) -> u32 {
        self.data.max_raw_memory_mb
    }

    pub fn sony_arw2_posterization_thr(&self) -> i32 {
        self.data.sony_arw2_posterization_thr
    }

    pub fn coolscan_nef_gamma(&self) -> f32 {
        self.data.coolscan_nef_gamma
    }

    pub fn p4shot_order(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.p4shot_order[0]).to_string_lossy() }
    }
}

// ============================================================================
// Color Data
// ============================================================================
pub struct ColorData<'a> {
    data: &'a sys::libraw_colordata_t,
}

impl<'a> ColorData<'a> {
    pub fn curve(&self) -> &[u16; 65536] {
        &self.data.curve
    }

    pub fn cblack(&self) -> &[u32; 4104] {
        &self.data.cblack
    }

    pub fn black(&self) -> u32 {
        self.data.black
    }

    pub fn data_maximum(&self) -> u32 {
        self.data.data_maximum
    }

    pub fn maximum(&self) -> u32 {
        self.data.maximum
    }

    pub fn linear_max(&self) -> &[i64; 4] {
        &self.data.linear_max
    }

    pub fn fmaximum(&self) -> f32 {
        self.data.fmaximum
    }

    pub fn fnorm(&self) -> f32 {
        self.data.fnorm
    }

    pub fn white(&self) -> &[[u16; 8]; 8] {
        &self.data.white
    }

    pub fn cam_mul(&self) -> &[f32; 4] {
        &self.data.cam_mul
    }

    pub fn pre_mul(&self) -> &[f32; 4] {
        &self.data.pre_mul
    }

    pub fn cmatrix(&self) -> &[[f32; 4]; 3] {
        &self.data.cmatrix
    }

    pub fn ccm(&self) -> &[[f32; 4]; 3] {
        &self.data.ccm
    }

    pub fn rgb_cam(&self) -> &[[f32; 4]; 3] {
        &self.data.rgb_cam
    }

    pub fn cam_xyz(&self) -> &[[f32; 3]; 4] {
        &self.data.cam_xyz
    }

    pub fn flash_used(&self) -> f32 {
        self.data.flash_used
    }

    pub fn canon_ev(&self) -> f32 {
        self.data.canon_ev
    }

    pub fn model2(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.model2[0]).to_string_lossy() }
    }

    pub fn unique_camera_model(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.UniqueCameraModel[0]).to_string_lossy() }
    }

    pub fn localized_camera_model(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.LocalizedCameraModel[0]).to_string_lossy() }
    }

    pub fn image_unique_id(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.ImageUniqueID[0]).to_string_lossy() }
    }

    pub fn raw_data_unique_id(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.RawDataUniqueID[0]).to_string_lossy() }
    }

    pub fn original_raw_filename(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.OriginalRawFileName[0]).to_string_lossy() }
    }

    pub fn profile(&self) -> Option<&[u8]> {
        if self.data.profile.is_null() || self.data.profile_length == 0 {
            None
        } else {
            Some(unsafe {
                std::slice::from_raw_parts(
                    self.data.profile as *const u8,
                    self.data.profile_length as usize,
                )
            })
        }
    }

    pub fn black_stat(&self) -> &[u32; 8] {
        &self.data.black_stat
    }

    pub fn dng_color(&self) -> &[DngColor; 2] {
        unsafe { std::mem::transmute(&self.data.dng_color) }
    }

    pub fn dng_levels(&self) -> DngLevels<'_> {
        DngLevels {
            data: &self.data.dng_levels,
        }
    }

    pub fn wb_coeffs(&self) -> &[[i32; 4]; 256] {
        &self.data.WB_Coeffs
    }

    pub fn wbct_coeffs(&self) -> &[[f32; 5]; 64] {
        &self.data.WBCT_Coeffs
    }

    pub fn as_shot_wb_applied(&self) -> bool {
        self.data.as_shot_wb_applied != 0
    }

    pub fn raw_bps(&self) -> u32 {
        self.data.raw_bps
    }

    pub fn exif_color_space(&self) -> i32 {
        self.data.ExifColorSpace
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DngColor {
    pub parsedfields: u32,
    pub illuminant: u16,
    pub calibration: [[f32; 4]; 4],
    pub colormatrix: [[f32; 3]; 4],
    pub forwardmatrix: [[f32; 4]; 3],
}

pub struct DngLevels<'a> {
    data: &'a sys::libraw_dng_levels_t,
}

impl<'a> DngLevels<'a> {
    pub fn parsedfields(&self) -> u32 {
        self.data.parsedfields
    }

    pub fn dng_cblack(&self) -> &[u32; 4104] {
        &self.data.dng_cblack
    }

    pub fn dng_black(&self) -> u32 {
        self.data.dng_black
    }

    pub fn dng_fcblack(&self) -> &[f32; 4104] {
        &self.data.dng_fcblack
    }

    pub fn dng_fblack(&self) -> f32 {
        self.data.dng_fblack
    }

    pub fn dng_whitelevel(&self) -> &[u32; 4] {
        &self.data.dng_whitelevel
    }

    pub fn default_crop(&self) -> &[u16; 4] {
        &self.data.default_crop
    }

    pub fn user_crop(&self) -> &[f32; 4] {
        &self.data.user_crop
    }

    pub fn preview_colorspace(&self) -> u32 {
        self.data.preview_colorspace
    }

    pub fn analogbalance(&self) -> &[f32; 4] {
        &self.data.analogbalance
    }

    pub fn asshotneutral(&self) -> &[f32; 4] {
        &self.data.asshotneutral
    }

    pub fn baseline_exposure(&self) -> f32 {
        self.data.baseline_exposure
    }

    pub fn linear_response_limit(&self) -> f32 {
        self.data.LinearResponseLimit
    }
}

// ============================================================================
// Other Metadata
// ============================================================================
pub struct ImgOther<'a> {
    data: &'a sys::libraw_imgother_t,
}

impl<'a> ImgOther<'a> {
    pub fn iso_speed(&self) -> f32 {
        self.data.iso_speed
    }

    pub fn shutter(&self) -> f32 {
        self.data.shutter
    }

    pub fn aperture(&self) -> f32 {
        self.data.aperture
    }

    pub fn focal_len(&self) -> f32 {
        self.data.focal_len
    }

    pub fn timestamp(&self) -> i64 {
        self.data.timestamp
    }

    pub fn datetime(&self) -> Option<DateTime<Local>> {
        let ts = self.data.timestamp;
        Local.timestamp_opt(ts, 0).single()
    }

    pub fn shot_order(&self) -> u32 {
        self.data.shot_order
    }

    pub fn gps_data(&self) -> &[u32; 32] {
        &self.data.gpsdata
    }

    pub fn parsed_gps(&self) -> GpsInfo<'_> {
        GpsInfo {
            data: &self.data.parsed_gps,
        }
    }

    pub fn desc(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.desc[0]).to_string_lossy() }
    }

    pub fn artist(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.artist[0]).to_string_lossy() }
    }

    pub fn analogbalance(&self) -> &[f32; 4] {
        &self.data.analogbalance
    }
}

pub struct GpsInfo<'a> {
    data: &'a sys::libraw_gps_info_t,
}

impl<'a> GpsInfo<'a> {
    pub fn latitude(&self) -> &[f32; 3] {
        &self.data.latitude
    }

    pub fn longitude(&self) -> &[f32; 3] {
        &self.data.longitude
    }

    pub fn gpstimestamp(&self) -> &[f32; 3] {
        &self.data.gpstimestamp
    }

    pub fn altitude(&self) -> f32 {
        self.data.altitude
    }

    pub fn altref(&self) -> i8 {
        self.data.altref
    }

    pub fn latref(&self) -> i8 {
        self.data.latref
    }

    pub fn longref(&self) -> i8 {
        self.data.longref
    }

    pub fn gpsstatus(&self) -> i8 {
        self.data.gpsstatus
    }

    pub fn gpsparsed(&self) -> bool {
        self.data.gpsparsed != 0
    }
}

// ============================================================================
// Thumbnail
// ============================================================================
pub struct Thumbnail<'a> {
    data: &'a sys::libraw_thumbnail_t,
}

impl<'a> Thumbnail<'a> {
    pub fn format(&self) -> u32 {
        self.data.tformat
    }

    pub fn width(&self) -> u16 {
        self.data.twidth
    }

    pub fn height(&self) -> u16 {
        self.data.theight
    }

    pub fn length(&self) -> u32 {
        self.data.tlength
    }

    pub fn colors(&self) -> i32 {
        self.data.tcolors
    }

    pub fn data(&self) -> Option<&[u8]> {
        if self.data.thumb.is_null() || self.data.tlength == 0 {
            None
        } else {
            Some(unsafe {
                std::slice::from_raw_parts(self.data.thumb as *const u8, self.data.tlength as usize)
            })
        }
    }
}

pub struct ThumbsList<'a> {
    data: &'a sys::libraw_thumbnail_list_t,
}

impl<'a> ThumbsList<'a> {
    pub fn count(&self) -> i32 {
        self.data.thumbcount
    }

    pub fn items(&self) -> &[ThumbnailItem] {
        let count = self.data.thumbcount.max(0) as usize;
        let count = count.min(8);
        unsafe { std::mem::transmute(&self.data.thumblist[..count]) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ThumbnailItem {
    pub format: u32,
    pub width: u16,
    pub height: u16,
    pub flip: u16,
    pub length: u32,
    pub misc: u32,
    pub offset: i64,
}

// ============================================================================
// Raw Data
// ============================================================================
pub struct RawData<'a> {
    data: &'a sys::libraw_rawdata_t,
}

impl<'a> RawData<'a> {
    pub fn iparams(&self) -> ImageParams<'_> {
        ImageParams {
            data: &self.data.iparams,
        }
    }

    pub fn sizes(&self) -> ImageSizes<'_> {
        ImageSizes {
            data: &self.data.sizes,
        }
    }

    pub fn ioparams(&self) -> InternalOutputParams<'_> {
        InternalOutputParams {
            data: &self.data.ioparams,
        }
    }

    pub fn color(&self) -> ColorData<'_> {
        ColorData {
            data: &self.data.color,
        }
    }

    pub fn raw_image(&self) -> Option<&[u16]> {
        let ptr = self.data.raw_image;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.raw_width as usize;
            let h = self.data.sizes.raw_height as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h) })
        }
    }

    pub fn color4_image(&self) -> Option<&[[u16; 4]]> {
        let ptr = self.data.color4_image;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.iwidth as usize;
            let h = self.data.sizes.iheight as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h) })
        }
    }

    pub fn color3_image(&self) -> Option<&[[u16; 3]]> {
        let ptr = self.data.color3_image;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.iwidth as usize;
            let h = self.data.sizes.iheight as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h) })
        }
    }

    pub fn float_image(&self) -> Option<&[f32]> {
        let ptr = self.data.float_image;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.iwidth as usize;
            let h = self.data.sizes.iheight as usize;
            let colors = self.data.iparams.colors as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h * colors) })
        }
    }

    pub fn float3_image(&self) -> Option<&[[f32; 3]]> {
        let ptr = self.data.float3_image;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.iwidth as usize;
            let h = self.data.sizes.iheight as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h) })
        }
    }

    pub fn float4_image(&self) -> Option<&[[f32; 4]]> {
        let ptr = self.data.float4_image;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.iwidth as usize;
            let h = self.data.sizes.iheight as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w * h) })
        }
    }

    pub fn ph1_cblack(&self) -> Option<&[[i16; 2]]> {
        let ptr = self.data.ph1_cblack;
        if ptr.is_null() {
            None
        } else {
            let h = self.data.sizes.raw_height as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, h) })
        }
    }

    pub fn ph1_rblack(&self) -> Option<&[[i16; 2]]> {
        let ptr = self.data.ph1_rblack;
        if ptr.is_null() {
            None
        } else {
            let w = self.data.sizes.raw_width as usize;
            Some(unsafe { std::slice::from_raw_parts(ptr, w) })
        }
    }
}

pub struct InternalOutputParams<'a> {
    data: &'a sys::libraw_internal_output_params_t,
}

impl<'a> InternalOutputParams<'a> {
    pub fn mix_green(&self) -> bool {
        self.data.mix_green != 0
    }

    pub fn raw_color(&self) -> bool {
        self.data.raw_color != 0
    }

    pub fn zero_is_bad(&self) -> bool {
        self.data.zero_is_bad != 0
    }

    pub fn shrink(&self) -> u16 {
        self.data.shrink
    }

    pub fn fuji_width(&self) -> u16 {
        self.data.fuji_width
    }
}

// ============================================================================
// Makernotes
// ============================================================================
pub struct Makernotes<'a> {
    data: &'a sys::libraw_makernotes_t,
}

impl<'a> Makernotes<'a> {
    pub fn canon(&self) -> CanonMakernotes<'_> {
        CanonMakernotes {
            data: &self.data.canon,
        }
    }

    pub fn nikon(&self) -> NikonMakernotes<'_> {
        NikonMakernotes {
            data: &self.data.nikon,
        }
    }

    pub fn hasselblad(&self) -> HasselbladMakernotes<'_> {
        HasselbladMakernotes {
            data: &self.data.hasselblad,
        }
    }

    pub fn fuji(&self) -> FujiInfo<'_> {
        FujiInfo {
            data: &self.data.fuji,
        }
    }

    pub fn olympus(&self) -> OlympusMakernotes<'_> {
        OlympusMakernotes {
            data: &self.data.olympus,
        }
    }

    pub fn sony(&self) -> SonyInfo<'_> {
        SonyInfo {
            data: &self.data.sony,
        }
    }

    pub fn kodak(&self) -> KodakMakernotes<'_> {
        KodakMakernotes {
            data: &self.data.kodak,
        }
    }

    pub fn panasonic(&self) -> PanasonicMakernotes<'_> {
        PanasonicMakernotes {
            data: &self.data.panasonic,
        }
    }

    pub fn pentax(&self) -> PentaxMakernotes<'_> {
        PentaxMakernotes {
            data: &self.data.pentax,
        }
    }

    pub fn phaseone(&self) -> PhaseOneMakernotes<'_> {
        PhaseOneMakernotes {
            data: &self.data.phaseone,
        }
    }

    pub fn ricoh(&self) -> RicohMakernotes<'_> {
        RicohMakernotes {
            data: &self.data.ricoh,
        }
    }

    pub fn samsung(&self) -> SamsungMakernotes<'_> {
        SamsungMakernotes {
            data: &self.data.samsung,
        }
    }

    pub fn common(&self) -> CommonMetadata<'_> {
        CommonMetadata {
            data: &self.data.common,
        }
    }
}

// ============================================================================
// Makernotes - Canon
// ============================================================================
pub struct CanonMakernotes<'a> {
    data: &'a sys::libraw_canon_makernotes_t,
}

impl<'a> CanonMakernotes<'a> {
    pub fn color_data_ver(&self) -> i32 {
        self.data.ColorDataVer
    }

    pub fn color_data_sub_ver(&self) -> i32 {
        self.data.ColorDataSubVer
    }

    pub fn specular_white_level(&self) -> i32 {
        self.data.SpecularWhiteLevel
    }

    pub fn normal_white_level(&self) -> i32 {
        self.data.NormalWhiteLevel
    }

    pub fn channel_black_level(&self) -> &[i32; 4] {
        &self.data.ChannelBlackLevel
    }

    pub fn average_black_level(&self) -> i32 {
        self.data.AverageBlackLevel
    }

    pub fn multishot(&self) -> &[u32; 4] {
        &self.data.multishot
    }

    pub fn metering_mode(&self) -> i16 {
        self.data.MeteringMode
    }

    pub fn spot_metering_mode(&self) -> i16 {
        self.data.SpotMeteringMode
    }

    pub fn flash_metering_mode(&self) -> u8 {
        self.data.FlashMeteringMode
    }

    pub fn flash_exposure_lock(&self) -> i16 {
        self.data.FlashExposureLock
    }

    pub fn exposure_mode(&self) -> i16 {
        self.data.ExposureMode
    }

    pub fn ae_setting(&self) -> i16 {
        self.data.AESetting
    }

    pub fn image_stabilization(&self) -> i16 {
        self.data.ImageStabilization
    }

    pub fn flash_mode(&self) -> i16 {
        self.data.FlashMode
    }

    pub fn flash_activity(&self) -> i16 {
        self.data.FlashActivity
    }

    pub fn flash_bits(&self) -> i16 {
        self.data.FlashBits
    }

    pub fn manual_flash_output(&self) -> i16 {
        self.data.ManualFlashOutput
    }

    pub fn flash_output(&self) -> i16 {
        self.data.FlashOutput
    }

    pub fn flash_guide_number(&self) -> i16 {
        self.data.FlashGuideNumber
    }

    pub fn continuous_drive(&self) -> i16 {
        self.data.ContinuousDrive
    }

    pub fn sensor_width(&self) -> i16 {
        self.data.SensorWidth
    }

    pub fn sensor_height(&self) -> i16 {
        self.data.SensorHeight
    }

    pub fn af_micro_adj_mode(&self) -> i32 {
        self.data.AFMicroAdjMode
    }

    pub fn af_micro_adj_value(&self) -> f32 {
        self.data.AFMicroAdjValue
    }

    pub fn makernotes_flip(&self) -> i16 {
        self.data.MakernotesFlip
    }

    pub fn record_mode(&self) -> i16 {
        self.data.RecordMode
    }

    pub fn sraw_quality(&self) -> i16 {
        self.data.SRAWQuality
    }

    pub fn wbi(&self) -> u32 {
        self.data.wbi
    }

    pub fn rf_lens_id(&self) -> i16 {
        self.data.RF_lensID
    }

    pub fn auto_lighting_optimizer(&self) -> i32 {
        self.data.AutoLightingOptimizer
    }

    pub fn highlight_tone_priority(&self) -> i32 {
        self.data.HighlightTonePriority
    }

    pub fn quality(&self) -> i16 {
        self.data.Quality
    }

    pub fn canon_log(&self) -> i32 {
        self.data.CanonLog
    }

    pub fn default_crop_absolute(&self) -> Area {
        unsafe { std::mem::transmute(self.data.DefaultCropAbsolute) }
    }

    pub fn recommended_image_area(&self) -> Area {
        unsafe { std::mem::transmute(self.data.RecommendedImageArea) }
    }

    pub fn left_optical_black(&self) -> Area {
        unsafe { std::mem::transmute(self.data.LeftOpticalBlack) }
    }

    pub fn upper_optical_black(&self) -> Area {
        unsafe { std::mem::transmute(self.data.UpperOpticalBlack) }
    }

    pub fn active_area(&self) -> Area {
        unsafe { std::mem::transmute(self.data.ActiveArea) }
    }

    pub fn iso_gain(&self) -> &[i16; 2] {
        &self.data.ISOgain
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Area {
    pub t: i16,
    pub l: i16,
    pub b: i16,
    pub r: i16,
}

// ============================================================================
// Makernotes - Nikon
// ============================================================================
pub struct NikonMakernotes<'a> {
    data: &'a sys::libraw_nikon_makernotes_t,
}

impl<'a> NikonMakernotes<'a> {
    pub fn exposure_bracket_value(&self) -> f64 {
        self.data.ExposureBracketValue
    }

    pub fn active_d_lighting(&self) -> u16 {
        self.data.ActiveDLighting
    }

    pub fn shooting_mode(&self) -> u16 {
        self.data.ShootingMode
    }

    pub fn image_stabilization(&self) -> &[u8; 7] {
        &self.data.ImageStabilization
    }

    pub fn vibration_reduction(&self) -> u8 {
        self.data.VibrationReduction
    }

    pub fn vr_mode(&self) -> u8 {
        self.data.VRMode
    }

    pub fn flash_setting(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.FlashSetting[0]).to_string_lossy() }
    }

    pub fn flash_type(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.FlashType[0]).to_string_lossy() }
    }

    pub fn flash_exposure_compensation(&self) -> &[u8; 4] {
        &self.data.FlashExposureCompensation
    }

    pub fn external_flash_exposure_comp(&self) -> &[u8; 4] {
        &self.data.ExternalFlashExposureComp
    }

    pub fn flash_exposure_bracket_value(&self) -> &[u8; 4] {
        &self.data.FlashExposureBracketValue
    }

    pub fn flash_mode(&self) -> u8 {
        self.data.FlashMode
    }

    pub fn flash_exposure_compensation2(&self) -> i8 {
        self.data.FlashExposureCompensation2
    }

    pub fn flash_exposure_compensation3(&self) -> i8 {
        self.data.FlashExposureCompensation3
    }

    pub fn flash_exposure_compensation4(&self) -> i8 {
        self.data.FlashExposureCompensation4
    }

    pub fn flash_source(&self) -> u8 {
        self.data.FlashSource
    }

    pub fn flash_firmware(&self) -> &[u8; 2] {
        &self.data.FlashFirmware
    }

    pub fn external_flash_flags(&self) -> u8 {
        self.data.ExternalFlashFlags
    }

    pub fn flash_control_commander_mode(&self) -> u8 {
        self.data.FlashControlCommanderMode
    }

    pub fn flash_output_and_compensation(&self) -> u8 {
        self.data.FlashOutputAndCompensation
    }

    pub fn flash_focal_length(&self) -> u8 {
        self.data.FlashFocalLength
    }

    pub fn flash_gn_distance(&self) -> u8 {
        self.data.FlashGNDistance
    }

    pub fn flash_group_control_mode(&self) -> &[u8; 4] {
        &self.data.FlashGroupControlMode
    }

    pub fn flash_group_output_and_compensation(&self) -> &[u8; 4] {
        &self.data.FlashGroupOutputAndCompensation
    }

    pub fn flash_color_filter(&self) -> u8 {
        self.data.FlashColorFilter
    }

    pub fn nef_compression(&self) -> u16 {
        self.data.NEFCompression
    }

    pub fn exposure_mode(&self) -> i32 {
        self.data.ExposureMode
    }

    pub fn exposure_program(&self) -> i32 {
        self.data.ExposureProgram
    }

    pub fn n_me_shots(&self) -> i32 {
        self.data.nMEshots
    }

    pub fn me_gain_on(&self) -> i32 {
        self.data.MEgainOn
    }

    pub fn me_wb(&self) -> &[f64; 4] {
        &self.data.ME_WB
    }

    pub fn af_fine_tune(&self) -> u8 {
        self.data.AFFineTune
    }

    pub fn af_fine_tune_index(&self) -> u8 {
        self.data.AFFineTuneIndex
    }

    pub fn af_fine_tune_adj(&self) -> i8 {
        self.data.AFFineTuneAdj
    }

    pub fn lens_data_version(&self) -> u32 {
        self.data.LensDataVersion
    }

    pub fn flash_info_version(&self) -> u32 {
        self.data.FlashInfoVersion
    }

    pub fn color_balance_version(&self) -> u32 {
        self.data.ColorBalanceVersion
    }

    pub fn key(&self) -> u8 {
        self.data.key
    }

    pub fn nef_bit_depth(&self) -> &[u16; 4] {
        &self.data.NEFBitDepth
    }

    pub fn high_speed_crop_format(&self) -> u16 {
        self.data.HighSpeedCropFormat
    }

    pub fn sensor_high_speed_crop(&self) -> SensorHighSpeedCrop {
        unsafe { std::mem::transmute(self.data.SensorHighSpeedCrop) }
    }

    pub fn sensor_width(&self) -> u16 {
        self.data.SensorWidth
    }

    pub fn sensor_height(&self) -> u16 {
        self.data.SensorHeight
    }

    pub fn active_d_lighting_alt(&self) -> u16 {
        self.data.Active_D_Lighting
    }

    pub fn shot_info_version(&self) -> u32 {
        self.data.ShotInfoVersion
    }

    pub fn makernotes_flip(&self) -> i16 {
        self.data.MakernotesFlip
    }

    pub fn roll_angle(&self) -> f64 {
        self.data.RollAngle
    }

    pub fn pitch_angle(&self) -> f64 {
        self.data.PitchAngle
    }

    pub fn yaw_angle(&self) -> f64 {
        self.data.YawAngle
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SensorHighSpeedCrop {
    pub cleft: u16,
    pub ctop: u16,
    pub cwidth: u16,
    pub cheight: u16,
}

// ============================================================================
// Makernotes - Hasselblad
// ============================================================================
pub struct HasselbladMakernotes<'a> {
    data: &'a sys::libraw_hasselblad_makernotes_t,
}

impl<'a> HasselbladMakernotes<'a> {
    pub fn base_iso(&self) -> i32 {
        self.data.BaseISO
    }

    pub fn gain(&self) -> f64 {
        self.data.Gain
    }

    pub fn sensor(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Sensor[0]).to_string_lossy() }
    }

    pub fn sensor_unit(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SensorUnit[0]).to_string_lossy() }
    }

    pub fn host_body(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.HostBody[0]).to_string_lossy() }
    }

    pub fn sensor_code(&self) -> i32 {
        self.data.SensorCode
    }

    pub fn sensor_sub_code(&self) -> i32 {
        self.data.SensorSubCode
    }

    pub fn coating_code(&self) -> i32 {
        self.data.CoatingCode
    }

    pub fn uncropped(&self) -> bool {
        self.data.uncropped != 0
    }

    pub fn capture_sequence_initiator(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.CaptureSequenceInitiator[0]).to_string_lossy() }
    }

    pub fn sensor_unit_connector(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SensorUnitConnector[0]).to_string_lossy() }
    }

    pub fn format(&self) -> i32 {
        self.data.format
    }

    pub fn n_ifd_cm(&self) -> &[i32; 2] {
        &self.data.nIFD_CM
    }

    pub fn recommended_crop(&self) -> &[i32; 2] {
        &self.data.RecommendedCrop
    }

    pub fn mn_color_matrix(&self) -> &[[f64; 3]; 4] {
        &self.data.mnColorMatrix
    }
}

// ============================================================================
// Makernotes - Fuji
// ============================================================================
pub struct FujiInfo<'a> {
    data: &'a sys::libraw_fuji_info_t,
}

impl<'a> FujiInfo<'a> {
    pub fn expo_mid_point_shift(&self) -> f32 {
        self.data.ExpoMidPointShift
    }

    pub fn dynamic_range(&self) -> u16 {
        self.data.DynamicRange
    }

    pub fn film_mode(&self) -> u16 {
        self.data.FilmMode
    }

    pub fn dynamic_range_setting(&self) -> u16 {
        self.data.DynamicRangeSetting
    }

    pub fn development_dynamic_range(&self) -> u16 {
        self.data.DevelopmentDynamicRange
    }

    pub fn auto_dynamic_range(&self) -> u16 {
        self.data.AutoDynamicRange
    }

    pub fn d_range_priority(&self) -> u16 {
        self.data.DRangePriority
    }

    pub fn d_range_priority_auto(&self) -> u16 {
        self.data.DRangePriorityAuto
    }

    pub fn d_range_priority_fixed(&self) -> u16 {
        self.data.DRangePriorityFixed
    }

    pub fn brightness_compensation(&self) -> f32 {
        self.data.BrightnessCompensation
    }

    pub fn focus_mode(&self) -> u16 {
        self.data.FocusMode
    }

    pub fn af_mode(&self) -> u16 {
        self.data.AFMode
    }

    pub fn focus_pixel(&self) -> &[u16; 2] {
        &self.data.FocusPixel
    }

    pub fn priority_settings(&self) -> u16 {
        self.data.PrioritySettings
    }

    pub fn focus_settings(&self) -> u32 {
        self.data.FocusSettings
    }

    pub fn af_c_settings(&self) -> u32 {
        self.data.AF_C_Settings
    }

    pub fn focus_warning(&self) -> u16 {
        self.data.FocusWarning
    }

    pub fn image_stabilization(&self) -> &[u16; 3] {
        &self.data.ImageStabilization
    }

    pub fn flash_mode(&self) -> u16 {
        self.data.FlashMode
    }

    pub fn wb_preset(&self) -> u16 {
        self.data.WB_Preset
    }

    pub fn shutter_type(&self) -> u16 {
        self.data.ShutterType
    }

    pub fn exr_mode(&self) -> u16 {
        self.data.ExrMode
    }

    pub fn get_macro(&self) -> u16 {
        self.data.Macro
    }

    pub fn rating(&self) -> u32 {
        self.data.Rating
    }

    pub fn crop_mode(&self) -> u16 {
        self.data.CropMode
    }

    pub fn serial_signature(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SerialSignature[0]).to_string_lossy() }
    }

    pub fn sensor_id(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SensorID[0]).to_string_lossy() }
    }

    pub fn raf_version(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.RAFVersion[0]).to_string_lossy() }
    }

    pub fn raf_data_generation(&self) -> i32 {
        self.data.RAFDataGeneration
    }

    pub fn raf_data_version(&self) -> u16 {
        self.data.RAFDataVersion
    }

    pub fn is_tsner_dts(&self) -> bool {
        self.data.isTSNERDTS != 0
    }

    pub fn drive_mode(&self) -> i16 {
        self.data.DriveMode
    }

    pub fn black_level(&self) -> &[u16; 9] {
        &self.data.BlackLevel
    }

    pub fn raf_data_image_size_table(&self) -> &[u32; 32] {
        &self.data.RAFData_ImageSizeTable
    }

    pub fn auto_bracketing(&self) -> i32 {
        self.data.AutoBracketing
    }

    pub fn sequence_number(&self) -> i32 {
        self.data.SequenceNumber
    }

    pub fn series_length(&self) -> i32 {
        self.data.SeriesLength
    }

    pub fn pixel_shift_offset(&self) -> &[f32; 2] {
        &self.data.PixelShiftOffset
    }

    pub fn image_count(&self) -> i32 {
        self.data.ImageCount
    }
}

// ============================================================================
// Makernotes - Olympus
// ============================================================================
pub struct OlympusMakernotes<'a> {
    data: &'a sys::libraw_olympus_makernotes_t,
}

impl<'a> OlympusMakernotes<'a> {
    pub fn camera_type2(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.CameraType2[0]).to_string_lossy() }
    }

    pub fn valid_bits(&self) -> u16 {
        self.data.ValidBits
    }

    pub fn sensor_calibration(&self) -> &[i32; 2] {
        &self.data.SensorCalibration
    }

    pub fn drive_mode(&self) -> &[u16; 5] {
        &self.data.DriveMode
    }

    pub fn color_space(&self) -> u16 {
        self.data.ColorSpace
    }

    pub fn focus_mode(&self) -> &[u16; 2] {
        &self.data.FocusMode
    }

    pub fn auto_focus(&self) -> u16 {
        self.data.AutoFocus
    }

    pub fn af_point(&self) -> u16 {
        self.data.AFPoint
    }

    pub fn af_areas(&self) -> &[u32; 64] {
        &self.data.AFAreas
    }

    pub fn af_point_selected(&self) -> &[f64; 5] {
        &self.data.AFPointSelected
    }

    pub fn af_result(&self) -> u16 {
        self.data.AFResult
    }

    pub fn af_fine_tune(&self) -> u8 {
        self.data.AFFineTune
    }

    pub fn af_fine_tune_adj(&self) -> &[i16; 3] {
        &self.data.AFFineTuneAdj
    }

    pub fn special_mode(&self) -> &[u32; 3] {
        &self.data.SpecialMode
    }

    pub fn zoom_step_count(&self) -> u16 {
        self.data.ZoomStepCount
    }

    pub fn focus_step_count(&self) -> u16 {
        self.data.FocusStepCount
    }

    pub fn focus_step_infinity(&self) -> u16 {
        self.data.FocusStepInfinity
    }

    pub fn focus_step_near(&self) -> u16 {
        self.data.FocusStepNear
    }

    pub fn focus_distance(&self) -> f64 {
        self.data.FocusDistance
    }

    pub fn aspect_frame(&self) -> &[u16; 4] {
        &self.data.AspectFrame
    }

    pub fn stacked_image(&self) -> &[u32; 2] {
        &self.data.StackedImage
    }

    pub fn is_live_nd(&self) -> bool {
        self.data.isLiveND != 0
    }

    pub fn live_nd_factor(&self) -> u32 {
        self.data.LiveNDfactor
    }

    pub fn panorama_mode(&self) -> u16 {
        self.data.Panorama_mode
    }

    pub fn panorama_frame_num(&self) -> u16 {
        self.data.Panorama_frameNum
    }
}

// ============================================================================
// Makernotes - Sony
// ============================================================================
pub struct SonyInfo<'a> {
    data: &'a sys::libraw_sony_info_t,
}

impl<'a> SonyInfo<'a> {
    pub fn camera_type(&self) -> u16 {
        self.data.CameraType
    }

    pub fn sony_0x9400_version(&self) -> u8 {
        self.data.Sony0x9400_version
    }

    pub fn sony_0x9400_release_mode2(&self) -> u8 {
        self.data.Sony0x9400_ReleaseMode2
    }

    pub fn sony_0x9400_sequence_image_number(&self) -> u32 {
        self.data.Sony0x9400_SequenceImageNumber
    }

    pub fn sony_0x9400_sequence_length1(&self) -> u8 {
        self.data.Sony0x9400_SequenceLength1
    }

    pub fn sony_0x9400_sequence_file_number(&self) -> u32 {
        self.data.Sony0x9400_SequenceFileNumber
    }

    pub fn sony_0x9400_sequence_length2(&self) -> u8 {
        self.data.Sony0x9400_SequenceLength2
    }

    pub fn af_area_mode_setting(&self) -> u8 {
        self.data.AFAreaModeSetting
    }

    pub fn af_area_mode(&self) -> u16 {
        self.data.AFAreaMode
    }

    pub fn flexible_spot_position(&self) -> &[u16; 2] {
        &self.data.FlexibleSpotPosition
    }

    pub fn af_point_selected(&self) -> u8 {
        self.data.AFPointSelected
    }

    pub fn af_point_selected_0x201e(&self) -> u8 {
        self.data.AFPointSelected_0x201e
    }

    pub fn n_af_points_used(&self) -> i16 {
        self.data.nAFPointsUsed
    }

    pub fn af_points_used(&self) -> &[u8; 10] {
        &self.data.AFPointsUsed
    }

    pub fn af_tracking(&self) -> u8 {
        self.data.AFTracking
    }

    pub fn af_type(&self) -> u8 {
        self.data.AFType
    }

    pub fn focus_location(&self) -> &[u16; 4] {
        &self.data.FocusLocation
    }

    pub fn focus_position(&self) -> u16 {
        self.data.FocusPosition
    }

    pub fn af_micro_adj_value(&self) -> i8 {
        self.data.AFMicroAdjValue
    }

    pub fn af_micro_adj_on(&self) -> i8 {
        self.data.AFMicroAdjOn
    }

    pub fn af_micro_adj_registered_lenses(&self) -> u8 {
        self.data.AFMicroAdjRegisteredLenses
    }

    pub fn variable_low_pass_filter(&self) -> u16 {
        self.data.VariableLowPassFilter
    }

    pub fn long_exposure_noise_reduction(&self) -> u32 {
        self.data.LongExposureNoiseReduction
    }

    pub fn high_iso_noise_reduction(&self) -> u16 {
        self.data.HighISONoiseReduction
    }

    pub fn hdr(&self) -> &[u16; 2] {
        &self.data.HDR
    }

    pub fn group2010(&self) -> u16 {
        self.data.group2010
    }

    pub fn group9050(&self) -> u16 {
        self.data.group9050
    }

    pub fn real_iso_offset(&self) -> u16 {
        self.data.real_iso_offset
    }

    pub fn metering_mode_offset(&self) -> u16 {
        self.data.MeteringMode_offset
    }

    pub fn exposure_program_offset(&self) -> u16 {
        self.data.ExposureProgram_offset
    }

    pub fn release_mode2_offset(&self) -> u16 {
        self.data.ReleaseMode2_offset
    }

    pub fn minolta_cam_id(&self) -> u32 {
        self.data.MinoltaCamID
    }

    pub fn firmware(&self) -> f32 {
        self.data.firmware
    }

    pub fn image_count3_offset(&self) -> u16 {
        self.data.ImageCount3_offset
    }

    pub fn image_count3(&self) -> u32 {
        self.data.ImageCount3
    }

    pub fn electronic_front_curtain_shutter(&self) -> u32 {
        self.data.ElectronicFrontCurtainShutter
    }

    pub fn metering_mode2(&self) -> u16 {
        self.data.MeteringMode2
    }

    pub fn sony_date_time(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SonyDateTime[0]).to_string_lossy() }
    }

    pub fn shot_number_since_power_up(&self) -> u32 {
        self.data.ShotNumberSincePowerUp
    }

    pub fn pixel_shift_group_prefix(&self) -> u16 {
        self.data.PixelShiftGroupPrefix
    }

    pub fn pixel_shift_group_id(&self) -> u32 {
        self.data.PixelShiftGroupID
    }

    pub fn n_shots_in_pixel_shift_group(&self) -> i8 {
        self.data.nShotsInPixelShiftGroup
    }

    pub fn num_in_pixel_shift_group(&self) -> i8 {
        self.data.numInPixelShiftGroup
    }

    pub fn prd_image_height(&self) -> u16 {
        self.data.prd_ImageHeight
    }

    pub fn prd_image_width(&self) -> u16 {
        self.data.prd_ImageWidth
    }

    pub fn prd_total_bps(&self) -> u16 {
        self.data.prd_Total_bps
    }

    pub fn prd_active_bps(&self) -> u16 {
        self.data.prd_Active_bps
    }

    pub fn prd_storage_method(&self) -> u16 {
        self.data.prd_StorageMethod
    }

    pub fn prd_bayer_pattern(&self) -> u16 {
        self.data.prd_BayerPattern
    }

    pub fn sony_raw_file_type(&self) -> u16 {
        self.data.SonyRawFileType
    }

    pub fn raw_file_type(&self) -> u16 {
        self.data.RAWFileType
    }

    pub fn raw_size_type(&self) -> u16 {
        self.data.RawSizeType
    }

    pub fn quality(&self) -> u32 {
        self.data.Quality
    }

    pub fn file_format(&self) -> u16 {
        self.data.FileFormat
    }

    pub fn meta_version(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.MetaVersion[0]).to_string_lossy() }
    }
}

// ============================================================================
// Makernotes - Kodak
// ============================================================================
pub struct KodakMakernotes<'a> {
    data: &'a sys::libraw_kodak_makernotes_t,
}

impl<'a> KodakMakernotes<'a> {
    pub fn black_level_top(&self) -> u16 {
        self.data.BlackLevelTop
    }

    pub fn black_level_bottom(&self) -> u16 {
        self.data.BlackLevelBottom
    }

    pub fn offset_left(&self) -> i16 {
        self.data.offset_left
    }

    pub fn offset_top(&self) -> i16 {
        self.data.offset_top
    }

    pub fn clip_black(&self) -> u16 {
        self.data.clipBlack
    }

    pub fn clip_white(&self) -> u16 {
        self.data.clipWhite
    }

    pub fn romm_cam_daylight(&self) -> &[[f32; 3]; 3] {
        &self.data.romm_camDaylight
    }

    pub fn romm_cam_tungsten(&self) -> &[[f32; 3]; 3] {
        &self.data.romm_camTungsten
    }

    pub fn romm_cam_fluorescent(&self) -> &[[f32; 3]; 3] {
        &self.data.romm_camFluorescent
    }

    pub fn romm_cam_flash(&self) -> &[[f32; 3]; 3] {
        &self.data.romm_camFlash
    }

    pub fn romm_cam_custom(&self) -> &[[f32; 3]; 3] {
        &self.data.romm_camCustom
    }

    pub fn romm_cam_auto(&self) -> &[[f32; 3]; 3] {
        &self.data.romm_camAuto
    }

    pub fn val018percent(&self) -> u16 {
        self.data.val018percent
    }

    pub fn val100percent(&self) -> u16 {
        self.data.val100percent
    }

    pub fn val170percent(&self) -> u16 {
        self.data.val170percent
    }

    pub fn maker_note_kodak8a(&self) -> i16 {
        self.data.MakerNoteKodak8a
    }

    pub fn iso_calibration_gain(&self) -> f32 {
        self.data.ISOCalibrationGain
    }

    pub fn analog_iso(&self) -> f32 {
        self.data.AnalogISO
    }
}

// ============================================================================
// Makernotes - Panasonic
// ============================================================================
pub struct PanasonicMakernotes<'a> {
    data: &'a sys::libraw_panasonic_makernotes_t,
}

impl<'a> PanasonicMakernotes<'a> {
    pub fn compression(&self) -> u16 {
        self.data.Compression
    }

    pub fn black_level_dim(&self) -> u16 {
        self.data.BlackLevelDim
    }

    pub fn black_level(&self) -> &[f32; 8] {
        &self.data.BlackLevel
    }

    pub fn multishot(&self) -> u32 {
        self.data.Multishot
    }

    pub fn gamma(&self) -> f32 {
        self.data.gamma
    }

    pub fn high_iso_multiplier(&self) -> &[i32; 3] {
        &self.data.HighISOMultiplier
    }

    pub fn focus_step_near(&self) -> i16 {
        self.data.FocusStepNear
    }

    pub fn focus_step_count(&self) -> i16 {
        self.data.FocusStepCount
    }

    pub fn zoom_position(&self) -> u32 {
        self.data.ZoomPosition
    }

    pub fn lens_manufacturer(&self) -> u32 {
        self.data.LensManufacturer
    }
}

// ============================================================================
// Makernotes - Pentax
// ============================================================================
pub struct PentaxMakernotes<'a> {
    data: &'a sys::libraw_pentax_makernotes_t,
}

impl<'a> PentaxMakernotes<'a> {
    pub fn drive_mode(&self) -> &[u8; 4] {
        &self.data.DriveMode
    }

    pub fn focus_mode(&self) -> &[u16; 2] {
        &self.data.FocusMode
    }

    pub fn af_point_selected(&self) -> &[u16; 2] {
        &self.data.AFPointSelected
    }

    pub fn af_point_selected_area(&self) -> u16 {
        self.data.AFPointSelected_Area
    }

    pub fn af_points_in_focus_version(&self) -> i32 {
        self.data.AFPointsInFocus_version
    }

    pub fn af_points_in_focus(&self) -> u32 {
        self.data.AFPointsInFocus
    }

    pub fn focus_position(&self) -> u16 {
        self.data.FocusPosition
    }

    pub fn af_adjustment(&self) -> i16 {
        self.data.AFAdjustment
    }

    pub fn af_point_mode(&self) -> u8 {
        self.data.AFPointMode
    }

    pub fn multi_exposure(&self) -> u8 {
        self.data.MultiExposure
    }

    pub fn quality(&self) -> u16 {
        self.data.Quality
    }
}

// ============================================================================
// Makernotes - Phase One
// ============================================================================
pub struct PhaseOneMakernotes<'a> {
    data: &'a sys::libraw_p1_makernotes_t,
}

impl<'a> PhaseOneMakernotes<'a> {
    pub fn software(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.Software[0]).to_string_lossy() }
    }

    pub fn system_type(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SystemType[0]).to_string_lossy() }
    }

    pub fn firmware_string(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.FirmwareString[0]).to_string_lossy() }
    }

    pub fn system_model(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.SystemModel[0]).to_string_lossy() }
    }
}

// ============================================================================
// Makernotes - Ricoh
// ============================================================================
pub struct RicohMakernotes<'a> {
    data: &'a sys::libraw_ricoh_makernotes_t,
}

impl<'a> RicohMakernotes<'a> {
    pub fn af_status(&self) -> u16 {
        self.data.AFStatus
    }

    pub fn af_area_x_position(&self) -> &[u32; 2] {
        &self.data.AFAreaXPosition
    }

    pub fn af_area_y_position(&self) -> &[u32; 2] {
        &self.data.AFAreaYPosition
    }

    pub fn af_area_mode(&self) -> u16 {
        self.data.AFAreaMode
    }

    pub fn sensor_width(&self) -> u32 {
        self.data.SensorWidth
    }

    pub fn sensor_height(&self) -> u32 {
        self.data.SensorHeight
    }

    pub fn cropped_image_width(&self) -> u32 {
        self.data.CroppedImageWidth
    }

    pub fn cropped_image_height(&self) -> u32 {
        self.data.CroppedImageHeight
    }

    pub fn wide_adapter(&self) -> u16 {
        self.data.WideAdapter
    }

    pub fn crop_mode(&self) -> u16 {
        self.data.CropMode
    }

    pub fn nd_filter(&self) -> u16 {
        self.data.NDFilter
    }

    pub fn auto_bracketing(&self) -> u16 {
        self.data.AutoBracketing
    }

    pub fn macro_mode(&self) -> u16 {
        self.data.MacroMode
    }

    pub fn flash_mode(&self) -> u16 {
        self.data.FlashMode
    }

    pub fn flash_exposure_comp(&self) -> f64 {
        self.data.FlashExposureComp
    }

    pub fn manual_flash_output(&self) -> f64 {
        self.data.ManualFlashOutput
    }
}

// ============================================================================
// Makernotes - Samsung
// ============================================================================
pub struct SamsungMakernotes<'a> {
    data: &'a sys::libraw_samsung_makernotes_t,
}

impl<'a> SamsungMakernotes<'a> {
    pub fn image_size_full(&self) -> &[u32; 4] {
        &self.data.ImageSizeFull
    }

    pub fn image_size_crop(&self) -> &[u32; 4] {
        &self.data.ImageSizeCrop
    }

    pub fn color_space(&self) -> &[i32; 2] {
        &self.data.ColorSpace
    }

    pub fn key(&self) -> &[u32; 11] {
        &self.data.key
    }

    pub fn digital_gain(&self) -> f64 {
        self.data.DigitalGain
    }

    pub fn device_type(&self) -> i32 {
        self.data.DeviceType
    }

    pub fn lens_firmware(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.LensFirmware[0]).to_string_lossy() }
    }
}

// ============================================================================
// Makernotes - Common Metadata
// ============================================================================
pub struct CommonMetadata<'a> {
    data: &'a sys::libraw_metadata_common_t,
}

impl<'a> CommonMetadata<'a> {
    pub fn flash_ec(&self) -> f32 {
        self.data.FlashEC
    }

    pub fn flash_gn(&self) -> f32 {
        self.data.FlashGN
    }

    pub fn camera_temperature(&self) -> f32 {
        self.data.CameraTemperature
    }

    pub fn sensor_temperature(&self) -> f32 {
        self.data.SensorTemperature
    }

    pub fn sensor_temperature2(&self) -> f32 {
        self.data.SensorTemperature2
    }

    pub fn lens_temperature(&self) -> f32 {
        self.data.LensTemperature
    }

    pub fn ambient_temperature(&self) -> f32 {
        self.data.AmbientTemperature
    }

    pub fn battery_temperature(&self) -> f32 {
        self.data.BatteryTemperature
    }

    pub fn exif_ambient_temperature(&self) -> f32 {
        self.data.exifAmbientTemperature
    }

    pub fn exif_humidity(&self) -> f32 {
        self.data.exifHumidity
    }

    pub fn exif_pressure(&self) -> f32 {
        self.data.exifPressure
    }

    pub fn exif_water_depth(&self) -> f32 {
        self.data.exifWaterDepth
    }

    pub fn exif_acceleration(&self) -> f32 {
        self.data.exifAcceleration
    }

    pub fn exif_camera_elevation_angle(&self) -> f32 {
        self.data.exifCameraElevationAngle
    }

    pub fn real_iso(&self) -> f32 {
        self.data.real_ISO
    }

    pub fn exif_exposure_index(&self) -> f32 {
        self.data.exifExposureIndex
    }

    pub fn color_space(&self) -> u16 {
        self.data.ColorSpace
    }

    pub fn firmware(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(&self.data.firmware[0]).to_string_lossy() }
    }

    pub fn exposure_calibration_shift(&self) -> f32 {
        self.data.ExposureCalibrationShift
    }

    pub fn af_data(&self) -> &[AfInfoItem; 4] {
        unsafe { std::mem::transmute(&self.data.afdata) }
    }

    pub fn af_count(&self) -> i32 {
        self.data.afcount
    }
}

#[repr(C)]
pub struct AfInfoItem {
    pub tag: u32,
    pub order: i16,
    pub version: u32,
    pub length: u32,
    pub data: *mut u8,
}

impl AfInfoItem {
    pub fn data(&self) -> Option<&[u8]> {
        if self.data.is_null() || self.length == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(self.data, self.length as usize) })
        }
    }
}
