//! Pixel types, image views, images and the format converter.

use crate::{NodeMap, Pylon, Result, bytes, c_string, check, handle, out, sys};
use std::ffi::CStr;
use std::ptr::NonNull;

/// Pixel type, a value of `Pylon::EPixelType`. A constant name is the pylon name with word
/// boundaries turned into underscores; the PFNC suffixes p, s and f stay attached to the bit depth.
/// Any other value can be used as well; pylon rejects values that are not pixel types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelType(pub u32);

impl PixelType {
    pub const UNDEFINED: Self = Self(sys::PYLON_PIXEL_TYPE_UNDEFINED);
    pub const MONO1_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_MONO1_PACKED);
    pub const MONO2_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_MONO2_PACKED);
    pub const MONO4_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_MONO4_PACKED);
    pub const MONO8: Self = Self(sys::PYLON_PIXEL_TYPE_MONO8);
    pub const MONO8_SIGNED: Self = Self(sys::PYLON_PIXEL_TYPE_MONO8_SIGNED);
    pub const MONO10: Self = Self(sys::PYLON_PIXEL_TYPE_MONO10);
    pub const MONO10_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_MONO10_PACKED);
    pub const MONO10P: Self = Self(sys::PYLON_PIXEL_TYPE_MONO10P);
    pub const MONO12: Self = Self(sys::PYLON_PIXEL_TYPE_MONO12);
    pub const MONO12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_MONO12_PACKED);
    pub const MONO12P: Self = Self(sys::PYLON_PIXEL_TYPE_MONO12P);
    pub const MONO16: Self = Self(sys::PYLON_PIXEL_TYPE_MONO16);
    pub const BAYER_GR8: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR8);
    pub const BAYER_RG8: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG8);
    pub const BAYER_GB8: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB8);
    pub const BAYER_BG8: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG8);
    pub const BAYER_GR10: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR10);
    pub const BAYER_RG10: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG10);
    pub const BAYER_GB10: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB10);
    pub const BAYER_BG10: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG10);
    pub const BAYER_GR12: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR12);
    pub const BAYER_RG12: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG12);
    pub const BAYER_GB12: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB12);
    pub const BAYER_BG12: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG12);
    pub const BAYER_GR12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR12_PACKED);
    pub const BAYER_RG12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG12_PACKED);
    pub const BAYER_GB12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB12_PACKED);
    pub const BAYER_BG12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG12_PACKED);
    pub const BAYER_GR10P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR10P);
    pub const BAYER_RG10P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG10P);
    pub const BAYER_GB10P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB10P);
    pub const BAYER_BG10P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG10P);
    pub const BAYER_GR12P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR12P);
    pub const BAYER_RG12P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG12P);
    pub const BAYER_GB12P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB12P);
    pub const BAYER_BG12P: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG12P);
    pub const BAYER_GR16: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GR16);
    pub const BAYER_RG16: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_RG16);
    pub const BAYER_GB16: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_GB16);
    pub const BAYER_BG16: Self = Self(sys::PYLON_PIXEL_TYPE_BAYER_BG16);
    pub const RGB8_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_RGB8_PACKED);
    pub const BGR8_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BGR8_PACKED);
    pub const RGBA8_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_RGBA8_PACKED);
    pub const BGRA8_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BGRA8_PACKED);
    pub const RGB10_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_RGB10_PACKED);
    pub const BGR10_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BGR10_PACKED);
    pub const RGB12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_RGB12_PACKED);
    pub const BGR12_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BGR12_PACKED);
    pub const RGB16_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_RGB16_PACKED);
    pub const BGR10V1_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BGR10V1_PACKED);
    pub const BGR10V2_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_BGR10V2_PACKED);
    pub const RGB12V1_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_RGB12V1_PACKED);
    pub const RGB8_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_RGB8_PLANAR);
    pub const RGB10_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_RGB10_PLANAR);
    pub const RGB12_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_RGB12_PLANAR);
    pub const RGB16_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_RGB16_PLANAR);
    pub const YUV411_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_YUV411_PACKED);
    pub const YUV422_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_YUV422_PACKED);
    pub const YUV444_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_YUV444_PACKED);
    pub const YUV422_YUYV_PACKED: Self = Self(sys::PYLON_PIXEL_TYPE_YUV422_YUYV_PACKED);
    pub const YUV444_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_YUV444_PLANAR);
    pub const YUV422_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_YUV422_PLANAR);
    pub const YUV420_PLANAR: Self = Self(sys::PYLON_PIXEL_TYPE_YUV420_PLANAR);
    pub const YCBCR420_8_YY_CBCR_SEMIPLANAR: Self =
        Self(sys::PYLON_PIXEL_TYPE_YCBCR420_8_YY_CBCR_SEMIPLANAR);
    pub const YCBCR422_8_YY_CBCR_SEMIPLANAR: Self =
        Self(sys::PYLON_PIXEL_TYPE_YCBCR422_8_YY_CBCR_SEMIPLANAR);
    pub const BICOLOR_RGBG8: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_RGBG8);
    pub const BICOLOR_BGRG8: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_BGRG8);
    pub const BICOLOR_RGBG10: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_RGBG10);
    pub const BICOLOR_RGBG10P: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_RGBG10P);
    pub const BICOLOR_BGRG10: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_BGRG10);
    pub const BICOLOR_BGRG10P: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_BGRG10P);
    pub const BICOLOR_RGBG12: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_RGBG12);
    pub const BICOLOR_RGBG12P: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_RGBG12P);
    pub const BICOLOR_BGRG12: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_BGRG12);
    pub const BICOLOR_BGRG12P: Self = Self(sys::PYLON_PIXEL_TYPE_BICOLOR_BGRG12P);
    pub const DOUBLE: Self = Self(sys::PYLON_PIXEL_TYPE_DOUBLE);
    pub const CONFIDENCE8: Self = Self(sys::PYLON_PIXEL_TYPE_CONFIDENCE8);
    pub const CONFIDENCE16: Self = Self(sys::PYLON_PIXEL_TYPE_CONFIDENCE16);
    pub const COORD3D_C8: Self = Self(sys::PYLON_PIXEL_TYPE_COORD3D_C8);
    pub const COORD3D_C16: Self = Self(sys::PYLON_PIXEL_TYPE_COORD3D_C16);
    pub const COORD3D_ABC32F: Self = Self(sys::PYLON_PIXEL_TYPE_COORD3D_ABC32F);
    pub const ERROR8: Self = Self(sys::PYLON_PIXEL_TYPE_ERROR8);
    pub const DATA8: Self = Self(sys::PYLON_PIXEL_TYPE_DATA8);
    pub const DATA8S: Self = Self(sys::PYLON_PIXEL_TYPE_DATA8S);
    pub const DATA16: Self = Self(sys::PYLON_PIXEL_TYPE_DATA16);
    pub const DATA16S: Self = Self(sys::PYLON_PIXEL_TYPE_DATA16S);
    pub const DATA32: Self = Self(sys::PYLON_PIXEL_TYPE_DATA32);
    pub const DATA32S: Self = Self(sys::PYLON_PIXEL_TYPE_DATA32S);
    pub const DATA32F: Self = Self(sys::PYLON_PIXEL_TYPE_DATA32F);
    pub const DATA64: Self = Self(sys::PYLON_PIXEL_TYPE_DATA64);
    pub const DATA64S: Self = Self(sys::PYLON_PIXEL_TYPE_DATA64S);
    pub const DATA64F: Self = Self(sys::PYLON_PIXEL_TYPE_DATA64F);

    /// Properties of the pixel type; fails for values that are not pixel types, e.g. UNDEFINED.
    pub fn info(self) -> Result<PixelTypeInfo> {
        // SAFETY: the out parameter is a valid PylonPixelTypeInfo.
        let info = out(|info| unsafe { sys::pylon_pixel_type_info(self.0, info) })?;
        // SAFETY: on PYLON_OK name is a non-NULL C string in static storage (pylon_shim.h).
        let name = unsafe { CStr::from_ptr(info.name) };
        Ok(PixelTypeInfo {
            name: name.to_str().unwrap_or_default(),
            bits_per_pixel: info.bits_per_pixel,
            bit_depth: info.bit_depth,
            samples_per_pixel: info.samples_per_pixel,
            plane_count: info.plane_count,
            is_mono: info.is_mono,
            is_bayer: info.is_bayer,
            is_packed: info.is_packed,
            has_alpha: info.has_alpha,
        })
    }

    /// `CPixelTypeMapper::GetPylonPixelTypeByName`, e.g. for the symbolic of the camera
    /// PixelFormat; None for unknown names.
    pub fn from_name(name: &str) -> Option<PixelType> {
        let name = c_string(name).ok()?;
        // SAFETY: name is a C string.
        let value = unsafe { sys::pylon_pixel_type_from_name(name.as_ptr()) };
        (value != sys::PYLON_PIXEL_TYPE_UNDEFINED).then_some(PixelType(value))
    }

    /// `Pylon::ComputeStride`: bytes per row. None where rows have no byte stride: rows that are
    /// not byte aligned and have no padding, planar YUV422 and YUV420; also for width 0 and for
    /// UNDEFINED.
    pub fn stride(self, width: u32, padding_x: usize) -> Result<Option<usize>> {
        // SAFETY: the out parameter is a valid usize.
        let stride = out(|stride| unsafe {
            sys::pylon_pixel_type_stride(self.0, width, padding_x, stride)
        })?;
        Ok((stride != 0).then_some(stride))
    }
}

/// Properties of a pixel type; those pylon leaves undefined for the type are 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelTypeInfo {
    /// SFNC 2.0 name; empty for the planar YUV types.
    pub name: &'static str,
    pub bits_per_pixel: u32,
    pub bit_depth: u32,
    pub samples_per_pixel: u32,
    pub plane_count: u32,
    /// `Pylon::IsMonoImage`; `Pylon::IsColorImage` is its complement.
    pub is_mono: bool,
    pub is_bayer: bool,
    pub is_packed: bool,
    pub has_alpha: bool,
}

/// Row order of an image, `Pylon::EImageOrientation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Orientation(sys::PylonImageOrientation);

impl Orientation {
    pub const TOP_DOWN: Self = Self(sys::PylonImageOrientation::PYLON_IMAGE_ORIENTATION_TOP_DOWN);
    pub const BOTTOM_UP: Self = Self(sys::PylonImageOrientation::PYLON_IMAGE_ORIENTATION_BOTTOM_UP);
}

/// Image memory with its layout, borrowed from a [`GrabResult`](crate::GrabResult), an [`Image`]
/// or the caller.
#[derive(Clone, Copy)]
pub struct ImageView<'a> {
    data: &'a [u8],
    pixel_type: PixelType,
    width: u32,
    height: u32,
    padding_x: usize,
    orientation: Orientation,
}

impl<'a> ImageView<'a> {
    /// Describes memory of the caller, e.g. to convert it. `padding_x` is the number of extra bytes
    /// at the end of each row. Conversions check that `data` is large enough for the layout.
    pub fn new(
        data: &'a [u8],
        pixel_type: PixelType,
        width: u32,
        height: u32,
        padding_x: usize,
        orientation: Orientation,
    ) -> Self {
        ImageView { data, pixel_type, width, height, padding_x, orientation }
    }

    /// View of memory reported by the shim; None for an invalid image.
    ///
    /// # Safety
    /// Unless `view.buffer` is NULL, it covers `view.size` bytes that stay valid and unchanged for
    /// `'a`.
    pub(crate) unsafe fn from_raw(view: &sys::PylonImageView) -> Option<Self> {
        if view.buffer.is_null() {
            return None;
        }
        Some(ImageView {
            // SAFETY: buffer is non-NULL and valid for 'a (caller contract).
            data: unsafe { bytes(view.buffer, view.size) },
            pixel_type: PixelType(view.pixel_type),
            width: view.width,
            height: view.height,
            padding_x: view.padding_x,
            orientation: Orientation(view.orientation),
        })
    }

    fn raw(&self) -> sys::PylonImageView {
        sys::PylonImageView {
            buffer: self.data.as_ptr().cast(),
            size: self.data.len(),
            pixel_type: self.pixel_type.0,
            width: self.width,
            height: self.height,
            padding_x: self.padding_x,
            orientation: self.orientation.0,
        }
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Extra bytes at the end of each row.
    pub fn padding_x(&self) -> usize {
        self.padding_x
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}

/// `Pylon::CPylonImage`, the target of conversions. Its buffer is reused while large enough.
pub struct Image {
    raw: NonNull<sys::PylonImage>,
    _pylon: Pylon,
}

// SAFETY: CPylonImage has no thread affinity; Image is not Sync because it is unsynchronized
// (pylon_shim.h Threads).
unsafe impl Send for Image {}

impl Image {
    /// Empty image.
    pub fn new(pylon: &Pylon) -> Result<Image> {
        // SAFETY: the out parameter is a valid handle pointer; pylon_image_create writes a
        // non-NULL handle on PYLON_OK (pylon_shim.h).
        let raw = unsafe { handle(|image| sys::pylon_image_create(image)) }?;
        Ok(Image { raw, _pylon: pylon.clone() })
    }

    /// The image as converted last; None before the first conversion and after a failed one.
    pub fn view(&self) -> Option<ImageView<'_>> {
        // SAFETY: raw is a live image.
        let view = unsafe { sys::pylon_image_view(self.raw.as_ptr()) };
        // SAFETY: the buffer stays valid until the next conversion into the image or its
        // destruction (pylon_shim.h); both need the image exclusively, so not during this borrow.
        unsafe { ImageView::from_raw(&view) }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // SAFETY: raw is a live image, destroyed once.
        unsafe { sys::pylon_image_destroy(self.raw.as_ptr()) };
    }
}

/// `Pylon::CImageFormatConverter`.
pub struct Converter {
    raw: NonNull<sys::PylonConverter>,
    _pylon: Pylon,
}

// SAFETY: CImageFormatConverter has no thread affinity; Converter is not Sync because it is
// unsynchronized (pylon_shim.h Threads).
unsafe impl Send for Converter {}

impl Converter {
    pub fn new(pylon: &Pylon) -> Result<Converter> {
        // SAFETY: the out parameter is a valid handle pointer; pylon_converter_create writes a
        // non-NULL handle on PYLON_OK (pylon_shim.h).
        let raw = unsafe { handle(|converter| sys::pylon_converter_create(converter)) }?;
        Ok(Converter { raw, _pylon: pylon.clone() })
    }

    /// Parameters of the converter: Gamma, MonoConversionMethod, OutputBitAlignment,
    /// OutputOrientation, OutputPaddingX, MaxNumThreads, ... InconvertibleEdgeHandling must stay
    /// SetZero, see [`convert`](Self::convert).
    pub fn node_map(&self) -> NodeMap<'_> {
        // SAFETY: raw is a live converter.
        let map = unsafe { sys::pylon_converter_node_map(self.raw.as_ptr()) };
        // SAFETY: the returned map is non-NULL and valid until the converter is destroyed
        // (pylon_shim.h), which needs ownership, so for the borrow of self.
        unsafe { NodeMap::from_raw(NonNull::new_unchecked(map)) }
    }

    /// `CImageFormatConverter::OutputPixelFormat`.
    pub fn set_output_pixel_type(&mut self, pixel_type: PixelType) -> Result<()> {
        // SAFETY: raw is a live converter.
        check(unsafe {
            sys::pylon_converter_set_output_pixel_type(self.raw.as_ptr(), pixel_type.0)
        })
    }

    /// True if converting the source under the current settings would only copy it, so the source
    /// can be used as it is.
    pub fn has_destination_format(&self, source: &ImageView<'_>) -> bool {
        let source = source.raw();
        // SAFETY: raw is a live converter; pylon reads only the layout of source.
        unsafe { sys::pylon_converter_has_destination_format(self.raw.as_ptr(), &source) }
    }

    /// Converts the source into the destination. On failure the destination is empty. Fails with
    /// [`ErrorKind::InvalidArgument`](crate::ErrorKind::InvalidArgument) when pylon would
    /// mishandle the conversion: for an InconvertibleEdgeHandling other than SetZero, the default,
    /// for the outputs YUV422_PLANAR and YUV420_PLANAR, and for source layouts too large for pylon
    /// to compute their size.
    pub fn convert(&mut self, destination: &mut Image, source: &ImageView<'_>) -> Result<()> {
        let source = source.raw();
        // SAFETY: both handles are live; the shim reads at most data.len() bytes of source
        // (pylon_shim.h).
        check(unsafe {
            sys::pylon_converter_convert(self.raw.as_ptr(), destination.raw.as_ptr(), &source)
        })
    }
}

impl Drop for Converter {
    fn drop(&mut self) {
        // SAFETY: raw is a live converter, destroyed once.
        unsafe { sys::pylon_converter_destroy(self.raw.as_ptr()) };
    }
}
