// pylon_shim.cpp - implementation of pylon_shim.h on top of the pylon C++ SDK.

#include "pylon_shim.h"

#include <pylon/PylonIncludes.h>

#include <cstdint>
#include <new>
#include <string>
#include <type_traits>

// Owned handles: each one holds exactly one pylon object.

struct PylonDeviceInfo {
    Pylon::CDeviceInfo info;
};

struct PylonCamera {
    Pylon::CInstantCamera camera;
};

struct PylonGrabResult {
    Pylon::CGrabResultPtr ptr;
};

struct PylonConverter {
    Pylon::CImageFormatConverter converter;
};

struct PylonImage {
    Pylon::CPylonImage image;
};

namespace {

// ---------------------------------------------------------------------------------------------
// Mirrored constants: verified here so that values cross the ABI with a plain cast.
// ---------------------------------------------------------------------------------------------

// Compares the 32-bit patterns that cross the ABI; EPixelType has int as underlying type on MSVC
// and long on GCC, so only its low 32 bits are meaningful.
template <class S, class P>
constexpr bool mirrors(S shim, P pylon) {
    return static_cast<std::uint32_t>(shim) == static_cast<std::uint32_t>(pylon);
}

static_assert(mirrors(PYLON_INFINITE, Pylon::waitForever));

static_assert(mirrors(PYLON_IMAGE_ORIENTATION_TOP_DOWN, Pylon::ImageOrientation_TopDown));
static_assert(mirrors(PYLON_IMAGE_ORIENTATION_BOTTOM_UP, Pylon::ImageOrientation_BottomUp));

static_assert(mirrors(PYLON_GRAB_STRATEGY_ONE_BY_ONE, Pylon::GrabStrategy_OneByOne));
static_assert(mirrors(PYLON_GRAB_STRATEGY_LATEST_IMAGE_ONLY, Pylon::GrabStrategy_LatestImageOnly));
static_assert(mirrors(PYLON_GRAB_STRATEGY_LATEST_IMAGES, Pylon::GrabStrategy_LatestImages));
static_assert(mirrors(PYLON_GRAB_STRATEGY_UPCOMING_IMAGE, Pylon::GrabStrategy_UpcomingImage));

static_assert(mirrors(PYLON_PAYLOAD_TYPE_UNDEFINED, Pylon::PayloadType_Undefined));
static_assert(mirrors(PYLON_PAYLOAD_TYPE_IMAGE, Pylon::PayloadType_Image));
static_assert(mirrors(PYLON_PAYLOAD_TYPE_RAW_DATA, Pylon::PayloadType_RawData));
static_assert(mirrors(PYLON_PAYLOAD_TYPE_FILE, Pylon::PayloadType_File));
static_assert(mirrors(PYLON_PAYLOAD_TYPE_CHUNK_DATA, Pylon::PayloadType_ChunkData));
static_assert(mirrors(PYLON_PAYLOAD_TYPE_GEN_DC, Pylon::PayloadType_GenDC));
static_assert(mirrors(PYLON_PAYLOAD_TYPE_DEVICE_SPECIFIC, Pylon::PayloadType_DeviceSpecific));

static_assert(mirrors(PYLON_NODE_TYPE_VALUE, GenApi::intfIValue));
static_assert(mirrors(PYLON_NODE_TYPE_BASE, GenApi::intfIBase));
static_assert(mirrors(PYLON_NODE_TYPE_INTEGER, GenApi::intfIInteger));
static_assert(mirrors(PYLON_NODE_TYPE_BOOLEAN, GenApi::intfIBoolean));
static_assert(mirrors(PYLON_NODE_TYPE_COMMAND, GenApi::intfICommand));
static_assert(mirrors(PYLON_NODE_TYPE_FLOAT, GenApi::intfIFloat));
static_assert(mirrors(PYLON_NODE_TYPE_STRING, GenApi::intfIString));
static_assert(mirrors(PYLON_NODE_TYPE_REGISTER, GenApi::intfIRegister));
static_assert(mirrors(PYLON_NODE_TYPE_CATEGORY, GenApi::intfICategory));
static_assert(mirrors(PYLON_NODE_TYPE_ENUMERATION, GenApi::intfIEnumeration));
static_assert(mirrors(PYLON_NODE_TYPE_ENUM_ENTRY, GenApi::intfIEnumEntry));
static_assert(mirrors(PYLON_NODE_TYPE_PORT, GenApi::intfIPort));

static_assert(mirrors(PYLON_ACCESS_MODE_NI, GenApi::NI));
static_assert(mirrors(PYLON_ACCESS_MODE_NA, GenApi::NA));
static_assert(mirrors(PYLON_ACCESS_MODE_WO, GenApi::WO));
static_assert(mirrors(PYLON_ACCESS_MODE_RO, GenApi::RO));
static_assert(mirrors(PYLON_ACCESS_MODE_RW, GenApi::RW));

static_assert(mirrors(PYLON_PIXEL_TYPE_UNDEFINED, Pylon::PixelType_Undefined));

static_assert(mirrors(PYLON_PIXEL_TYPE_MONO1_PACKED, Pylon::PixelType_Mono1packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO2_PACKED, Pylon::PixelType_Mono2packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO4_PACKED, Pylon::PixelType_Mono4packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO8, Pylon::PixelType_Mono8));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO8_SIGNED, Pylon::PixelType_Mono8signed));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO10, Pylon::PixelType_Mono10));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO10_PACKED, Pylon::PixelType_Mono10packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO10P, Pylon::PixelType_Mono10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO12, Pylon::PixelType_Mono12));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO12_PACKED, Pylon::PixelType_Mono12packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO12P, Pylon::PixelType_Mono12p));
static_assert(mirrors(PYLON_PIXEL_TYPE_MONO16, Pylon::PixelType_Mono16));

static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR8, Pylon::PixelType_BayerGR8));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG8, Pylon::PixelType_BayerRG8));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB8, Pylon::PixelType_BayerGB8));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG8, Pylon::PixelType_BayerBG8));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR10, Pylon::PixelType_BayerGR10));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG10, Pylon::PixelType_BayerRG10));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB10, Pylon::PixelType_BayerGB10));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG10, Pylon::PixelType_BayerBG10));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR12, Pylon::PixelType_BayerGR12));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG12, Pylon::PixelType_BayerRG12));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB12, Pylon::PixelType_BayerGB12));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG12, Pylon::PixelType_BayerBG12));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR12_PACKED, Pylon::PixelType_BayerGR12Packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG12_PACKED, Pylon::PixelType_BayerRG12Packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB12_PACKED, Pylon::PixelType_BayerGB12Packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG12_PACKED, Pylon::PixelType_BayerBG12Packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR10P, Pylon::PixelType_BayerGR10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG10P, Pylon::PixelType_BayerRG10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB10P, Pylon::PixelType_BayerGB10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG10P, Pylon::PixelType_BayerBG10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR12P, Pylon::PixelType_BayerGR12p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG12P, Pylon::PixelType_BayerRG12p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB12P, Pylon::PixelType_BayerGB12p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG12P, Pylon::PixelType_BayerBG12p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GR16, Pylon::PixelType_BayerGR16));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_RG16, Pylon::PixelType_BayerRG16));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_GB16, Pylon::PixelType_BayerGB16));
static_assert(mirrors(PYLON_PIXEL_TYPE_BAYER_BG16, Pylon::PixelType_BayerBG16));

static_assert(mirrors(PYLON_PIXEL_TYPE_RGB8_PACKED, Pylon::PixelType_RGB8packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BGR8_PACKED, Pylon::PixelType_BGR8packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGBA8_PACKED, Pylon::PixelType_RGBA8packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BGRA8_PACKED, Pylon::PixelType_BGRA8packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB10_PACKED, Pylon::PixelType_RGB10packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BGR10_PACKED, Pylon::PixelType_BGR10packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB12_PACKED, Pylon::PixelType_RGB12packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BGR12_PACKED, Pylon::PixelType_BGR12packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB16_PACKED, Pylon::PixelType_RGB16packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BGR10V1_PACKED, Pylon::PixelType_BGR10V1packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_BGR10V2_PACKED, Pylon::PixelType_BGR10V2packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB12V1_PACKED, Pylon::PixelType_RGB12V1packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB8_PLANAR, Pylon::PixelType_RGB8planar));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB10_PLANAR, Pylon::PixelType_RGB10planar));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB12_PLANAR, Pylon::PixelType_RGB12planar));
static_assert(mirrors(PYLON_PIXEL_TYPE_RGB16_PLANAR, Pylon::PixelType_RGB16planar));

static_assert(mirrors(PYLON_PIXEL_TYPE_YUV411_PACKED, Pylon::PixelType_YUV411packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_YUV422_PACKED, Pylon::PixelType_YUV422packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_YUV444_PACKED, Pylon::PixelType_YUV444packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_YUV422_YUYV_PACKED, Pylon::PixelType_YUV422_YUYV_Packed));
static_assert(mirrors(PYLON_PIXEL_TYPE_YUV444_PLANAR, Pylon::PixelType_YUV444planar));
static_assert(mirrors(PYLON_PIXEL_TYPE_YUV422_PLANAR, Pylon::PixelType_YUV422planar));
static_assert(mirrors(PYLON_PIXEL_TYPE_YUV420_PLANAR, Pylon::PixelType_YUV420planar));
static_assert(mirrors(PYLON_PIXEL_TYPE_YCBCR420_8_YY_CBCR_SEMIPLANAR, Pylon::PixelType_YCbCr420_8_YY_CbCr_Semiplanar));
static_assert(mirrors(PYLON_PIXEL_TYPE_YCBCR422_8_YY_CBCR_SEMIPLANAR, Pylon::PixelType_YCbCr422_8_YY_CbCr_Semiplanar));

static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_RGBG8, Pylon::PixelType_BiColorRGBG8));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_BGRG8, Pylon::PixelType_BiColorBGRG8));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_RGBG10, Pylon::PixelType_BiColorRGBG10));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_RGBG10P, Pylon::PixelType_BiColorRGBG10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_BGRG10, Pylon::PixelType_BiColorBGRG10));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_BGRG10P, Pylon::PixelType_BiColorBGRG10p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_RGBG12, Pylon::PixelType_BiColorRGBG12));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_RGBG12P, Pylon::PixelType_BiColorRGBG12p));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_BGRG12, Pylon::PixelType_BiColorBGRG12));
static_assert(mirrors(PYLON_PIXEL_TYPE_BICOLOR_BGRG12P, Pylon::PixelType_BiColorBGRG12p));

static_assert(mirrors(PYLON_PIXEL_TYPE_DOUBLE, Pylon::PixelType_Double));
static_assert(mirrors(PYLON_PIXEL_TYPE_CONFIDENCE8, Pylon::PixelType_Confidence8));
static_assert(mirrors(PYLON_PIXEL_TYPE_CONFIDENCE16, Pylon::PixelType_Confidence16));
static_assert(mirrors(PYLON_PIXEL_TYPE_COORD3D_C8, Pylon::PixelType_Coord3D_C8));
static_assert(mirrors(PYLON_PIXEL_TYPE_COORD3D_C16, Pylon::PixelType_Coord3D_C16));
static_assert(mirrors(PYLON_PIXEL_TYPE_COORD3D_ABC32F, Pylon::PixelType_Coord3D_ABC32f));
static_assert(mirrors(PYLON_PIXEL_TYPE_ERROR8, Pylon::PixelType_Error8));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA8, Pylon::PixelType_Data8));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA8S, Pylon::PixelType_Data8s));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA16, Pylon::PixelType_Data16));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA16S, Pylon::PixelType_Data16s));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA32, Pylon::PixelType_Data32));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA32S, Pylon::PixelType_Data32s));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA32F, Pylon::PixelType_Data32f));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA64, Pylon::PixelType_Data64));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA64S, Pylon::PixelType_Data64s));
static_assert(mirrors(PYLON_PIXEL_TYPE_DATA64F, Pylon::PixelType_Data64f));

// ---------------------------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------------------------

thread_local std::string last_error_message;

PylonStatus fail(PylonStatus status, const char* message) noexcept {
    last_error_message = message;
    return status;
}

// Maps the exception in flight to a status.
PylonStatus current_exception_status() noexcept {
    try {
        throw;
    } catch (const GenICam::BadAllocException& e) {
        return fail(PYLON_ERROR_BAD_ALLOC, e.GetDescription());
    } catch (const GenICam::InvalidArgumentException& e) {
        return fail(PYLON_ERROR_INVALID_ARGUMENT, e.GetDescription());
    } catch (const GenICam::OutOfRangeException& e) {
        return fail(PYLON_ERROR_OUT_OF_RANGE, e.GetDescription());
    } catch (const GenICam::PropertyException& e) {
        return fail(PYLON_ERROR_PROPERTY, e.GetDescription());
    } catch (const GenICam::RuntimeException& e) {
        return fail(PYLON_ERROR_RUNTIME, e.GetDescription());
    } catch (const GenICam::LogicalErrorException& e) {
        return fail(PYLON_ERROR_LOGICAL, e.GetDescription());
    } catch (const GenICam::AccessException& e) {
        return fail(PYLON_ERROR_ACCESS, e.GetDescription());
    } catch (const GenICam::TimeoutException& e) {
        return fail(PYLON_ERROR_TIMEOUT, e.GetDescription());
    } catch (const GenICam::DynamicCastException& e) {
        return fail(PYLON_ERROR_DYNAMIC_CAST, e.GetDescription());
    } catch (const GenICam::GenericException& e) {
        return fail(PYLON_ERROR_GENERIC, e.GetDescription());
    } catch (const std::bad_alloc& e) {
        return fail(PYLON_ERROR_BAD_ALLOC, e.what());
    } catch (const std::exception& e) {
        return fail(PYLON_ERROR_UNKNOWN, e.what());
    } catch (...) {
        return fail(PYLON_ERROR_UNKNOWN, "unknown exception");
    }
}

// Runs body and turns any exception into a status; the only place where exceptions become
// statuses.
template <class Body>
PylonStatus guard(Body&& body) noexcept {
    try {
        body();
        return PYLON_OK;
    } catch (...) {
        return current_exception_status();
    }
}

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

void emit(PylonStringCallback cb, void* ctx, const GenICam::gcstring& text) {
    cb(ctx, text.c_str(), text.size());
}

PylonNodeMap* wrap(GenApi::INodeMap& map) {
    return reinterpret_cast<PylonNodeMap*>(&map);
}

GenApi::INodeMap* unwrap(PylonNodeMap* map) {
    return reinterpret_cast<GenApi::INodeMap*>(map);
}

PylonNode* wrap(GenApi::INode* node) {
    return reinterpret_cast<PylonNode*>(node);
}

GenApi::INode* unwrap(PylonNode* node) {
    return reinterpret_cast<GenApi::INode*>(node);
}

// EPixelType has int as underlying type on MSVC and long on GCC; uint32_t carries its bit pattern.
std::uint32_t from_pixel_type(Pylon::EPixelType type) {
    return static_cast<std::uint32_t>(type);
}

Pylon::EPixelType to_pixel_type(std::uint32_t value) {
    using Underlying = std::underlying_type_t<Pylon::EPixelType>;
    return value == PYLON_PIXEL_TYPE_UNDEFINED
               ? Pylon::PixelType_Undefined
               : static_cast<Pylon::EPixelType>(static_cast<Underlying>(value));
}

// The node as GenApi interface T; this is where the node type contract is checked.
template <class T>
T& as(PylonNode* handle) {
    GenApi::INode* node = unwrap(handle);
    if (T* typed = dynamic_cast<T*>(node)) {
        return *typed;
    }
    throw DYNAMICCAST_EXCEPTION("Node %s does not implement the requested interface",
                                node->GetName().c_str());
}

PylonGrabResult* new_grab_result(const Pylon::CGrabResultPtr& ptr) {
    return ptr.IsValid() ? new PylonGrabResult{ptr} : nullptr;
}

// Pixel type property, or 0 where pylon leaves it undefined for the type.
template <class Property>
std::uint32_t defined_or_zero(Property property) {
    try {
        return property();
    } catch (const GenICam::InvalidArgumentException&) {
        return 0;
    }
}

PylonImageView view(const Pylon::IImage& image) {
    return {
        image.GetBuffer(),
        image.GetImageSize(),
        from_pixel_type(image.GetPixelType()),
        image.GetWidth(),
        image.GetHeight(),
        image.GetPaddingX(),
        static_cast<PylonImageOrientation>(image.GetOrientation()),
    };
}

GenApi::INodeMap& camera_node_map(Pylon::CInstantCamera& camera, PylonNodeMapKind kind) {
    switch (kind) {
    case PYLON_NODE_MAP_DEVICE:
        return camera.GetNodeMap();
    case PYLON_NODE_MAP_TRANSPORT_LAYER:
        return camera.GetTLNodeMap();
    case PYLON_NODE_MAP_STREAM_GRABBER:
        return camera.GetStreamGrabberNodeMap();
    case PYLON_NODE_MAP_EVENT_GRABBER:
        return camera.GetEventGrabberNodeMap();
    case PYLON_NODE_MAP_INSTANT_CAMERA:
        return camera.GetInstantCameraNodeMap();
    }
    throw INVALID_ARGUMENT_EXCEPTION("Invalid node map kind %d", static_cast<int>(kind));
}

Pylon::CConfigurationEventHandler* new_configuration(PylonConfiguration configuration) {
    switch (configuration) {
    case PYLON_CONFIGURATION_NONE:
        return nullptr;
    case PYLON_CONFIGURATION_ACQUIRE_CONTINUOUS:
        return new Pylon::CAcquireContinuousConfiguration;
    case PYLON_CONFIGURATION_ACQUIRE_SINGLE_FRAME:
        return new Pylon::CAcquireSingleFrameConfiguration;
    case PYLON_CONFIGURATION_SOFTWARE_TRIGGER:
        return new Pylon::CSoftwareTriggerConfiguration;
    }
    throw INVALID_ARGUMENT_EXCEPTION("Invalid configuration %d", static_cast<int>(configuration));
}

GenICam::gcstring node_text(const GenApi::INode& node, PylonNodeText text) {
    switch (text) {
    case PYLON_NODE_TEXT_NAME:
        return node.GetName();
    case PYLON_NODE_TEXT_DISPLAY_NAME:
        return node.GetDisplayName();
    case PYLON_NODE_TEXT_TOOL_TIP:
        return node.GetToolTip();
    case PYLON_NODE_TEXT_DESCRIPTION:
        return node.GetDescription();
    }
    throw INVALID_ARGUMENT_EXCEPTION("Invalid node text %d", static_cast<int>(text));
}

// A C callback as the function GenApi::Register expects; GenApi tests it before each call.
struct NodeCallback {
    PylonNodeCallback cb;
    void* ctx;

    explicit operator bool() const {
        return cb != nullptr;
    }

    void operator()(GenApi::INode* node) const {
        cb(ctx, wrap(node));
    }
};

} // namespace

extern "C" {

// ---------------------------------------------------------------------------------------------
// Status and runtime
// ---------------------------------------------------------------------------------------------

const char* pylon_last_error(void) {
    return last_error_message.c_str();
}

PylonStatus pylon_initialize(void) {
    return guard([] { Pylon::PylonInitialize(); });
}

PylonStatus pylon_terminate(void) {
    return guard([] { Pylon::PylonTerminate(); });
}

PylonVersion pylon_version(void) {
    unsigned int major = 0;
    unsigned int minor = 0;
    unsigned int subminor = 0;
    unsigned int build = 0;
    Pylon::GetPylonVersion(&major, &minor, &subminor, &build);
    return {major, minor, subminor, build};
}

// ---------------------------------------------------------------------------------------------
// Pixel types
// ---------------------------------------------------------------------------------------------

PylonStatus pylon_pixel_type_info(uint32_t pixel_type, PylonPixelTypeInfo* out) {
    return guard([&] {
        const Pylon::EPixelType type = to_pixel_type(pixel_type);
        *out = {
            Pylon::CPixelTypeMapper::GetNameByPixelType(type, Pylon::SFNCVersion_2_0),
            Pylon::BitPerPixel(type), // defined for every pixel type, throws for other values
            defined_or_zero([type] { return Pylon::BitDepth(type); }),
            defined_or_zero([type] { return Pylon::SamplesPerPixel(type); }),
            Pylon::PlaneCount(type),
            Pylon::IsMonoImage(type),
            Pylon::IsBayer(type),
            Pylon::IsPacked(type),
            Pylon::HasAlpha(type),
        };
    });
}

uint32_t pylon_pixel_type_from_name(const char* name) {
    return from_pixel_type(Pylon::CPixelTypeMapper::GetPylonPixelTypeByName(name));
}

PylonStatus pylon_pixel_type_stride(uint32_t pixel_type, uint32_t width, size_t padding_x,
                                    size_t* out) {
    return guard([&] {
        size_t stride = 0;
        *out = Pylon::ComputeStride(stride, to_pixel_type(pixel_type), width, padding_x) ? stride : 0;
    });
}

// ---------------------------------------------------------------------------------------------
// Device info
// ---------------------------------------------------------------------------------------------

PylonStatus pylon_device_info_create(PylonDeviceInfo** out) {
    return guard([&] { *out = new PylonDeviceInfo{}; });
}

void pylon_device_info_destroy(PylonDeviceInfo* info) {
    delete info;
}

PylonStatus pylon_device_info_get(const PylonDeviceInfo* info, const char* key,
                                  PylonStringCallback cb, void* ctx) {
    return guard([&] {
        Pylon::String_t value;
        if (info->info.GetPropertyValue(key, value)) {
            emit(cb, ctx, value);
        }
    });
}

PylonStatus pylon_device_info_set(PylonDeviceInfo* info, const char* key, const char* value) {
    return guard([&] { info->info.SetPropertyValue(key, value); });
}

PylonStatus pylon_device_info_keys(const PylonDeviceInfo* info, PylonStringCallback cb,
                                   void* ctx) {
    return guard([&] {
        Pylon::StringList_t keys;
        info->info.GetPropertyNames(keys);
        for (const Pylon::String_t& key : keys) {
            emit(cb, ctx, key);
        }
    });
}

PylonStatus pylon_enumerate_devices(const PylonDeviceInfo* const* filters, size_t filter_count,
                                    PylonDeviceInfoCallback cb, void* ctx) {
    return guard([&] {
        Pylon::CTlFactory& factory = Pylon::CTlFactory::GetInstance();
        Pylon::DeviceInfoList_t devices;
        if (filter_count == 0) {
            factory.EnumerateDevices(devices);
        } else {
            Pylon::DeviceInfoList_t filter;
            for (size_t i = 0; i < filter_count; ++i) {
                filter.push_back(filters[i]->info);
            }
            factory.EnumerateDevices(devices, filter);
        }
        for (const Pylon::CDeviceInfo& device : devices) {
            cb(ctx, new PylonDeviceInfo{device});
        }
    });
}

// ---------------------------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------------------------

PylonStatus pylon_camera_create(const PylonDeviceInfo* info, PylonCamera** out) {
    return guard([&] {
        Pylon::CTlFactory& factory = Pylon::CTlFactory::GetInstance();
        // new allocates before the device is created, and attaching a closed device does not
        // throw, so the device is never left unowned.
        *out = new PylonCamera{Pylon::CInstantCamera(
            info ? factory.CreateDevice(info->info) : factory.CreateFirstDevice())};
    });
}

void pylon_camera_destroy(PylonCamera* camera) {
    delete camera;
}

PylonStatus pylon_camera_device_info(const PylonCamera* camera, PylonDeviceInfo** out) {
    return guard([&] { *out = new PylonDeviceInfo{camera->camera.GetDeviceInfo()}; });
}

PylonStatus pylon_camera_open(PylonCamera* camera) {
    return guard([&] { camera->camera.Open(); });
}

void pylon_camera_close(PylonCamera* camera) {
    camera->camera.Close();
}

bool pylon_camera_is_open(const PylonCamera* camera) {
    return camera->camera.IsOpen();
}

bool pylon_camera_is_device_removed(const PylonCamera* camera) {
    return camera->camera.IsCameraDeviceRemoved();
}

PylonStatus pylon_camera_node_map(PylonCamera* camera, PylonNodeMapKind kind, PylonNodeMap** out) {
    return guard([&] { *out = wrap(camera_node_map(camera->camera, kind)); });
}

PylonStatus pylon_camera_set_configuration(PylonCamera* camera, PylonConfiguration configuration) {
    return guard([&] {
        camera->camera.RegisterConfiguration(new_configuration(configuration),
                                             Pylon::RegistrationMode_ReplaceAll,
                                             Pylon::Cleanup_Delete);
    });
}

PylonStatus pylon_camera_start_grabbing(PylonCamera* camera, PylonGrabStrategy strategy,
                                        size_t max_images) {
    return guard([&] {
        // Opening first marks the camera as opened by the user, so stopping never closes it.
        camera->camera.Open();
        const auto grab_strategy = static_cast<Pylon::EGrabStrategy>(strategy);
        if (max_images == 0) {
            camera->camera.StartGrabbing(grab_strategy);
        } else {
            camera->camera.StartGrabbing(max_images, grab_strategy);
        }
    });
}

void pylon_camera_stop_grabbing(PylonCamera* camera) {
    camera->camera.StopGrabbing();
}

bool pylon_camera_is_grabbing(const PylonCamera* camera) {
    return camera->camera.IsGrabbing();
}

PylonStatus pylon_camera_retrieve_result(PylonCamera* camera, uint32_t timeout_ms,
                                         PylonGrabResult** out) {
    return guard([&] {
        Pylon::CGrabResultPtr ptr;
        camera->camera.RetrieveResult(timeout_ms, ptr, Pylon::TimeoutHandling_Return);
        *out = new_grab_result(ptr);
    });
}

PylonStatus pylon_camera_grab_one(PylonCamera* camera, uint32_t timeout_ms,
                                  PylonGrabResult** out) {
    return guard([&] {
        camera->camera.Open(); // see pylon_camera_start_grabbing
        Pylon::CGrabResultPtr ptr;
        camera->camera.GrabOne(timeout_ms, ptr, Pylon::TimeoutHandling_Return);
        *out = new_grab_result(ptr);
    });
}

PylonStatus pylon_camera_can_wait_for_frame_trigger_ready(const PylonCamera* camera, bool* out) {
    return guard([&] { *out = camera->camera.CanWaitForFrameTriggerReady(); });
}

PylonStatus pylon_camera_wait_for_frame_trigger_ready(PylonCamera* camera, uint32_t timeout_ms,
                                                      bool* out) {
    return guard([&] {
        *out = camera->camera.WaitForFrameTriggerReady(timeout_ms, Pylon::TimeoutHandling_Return);
    });
}

PylonStatus pylon_camera_execute_software_trigger(PylonCamera* camera) {
    return guard([&] { camera->camera.ExecuteSoftwareTrigger(); });
}

// ---------------------------------------------------------------------------------------------
// Grab result
// ---------------------------------------------------------------------------------------------

void pylon_grab_result_destroy(PylonGrabResult* result) {
    delete result;
}

PylonStatus pylon_grab_result_info(const PylonGrabResult* result, PylonGrabResultInfo* out) {
    return guard([&] {
        const Pylon::CGrabResultPtr& ptr = result->ptr;
        *out = {
            ptr->GrabSucceeded(),
            ptr->GetErrorCode(),
            static_cast<PylonPayloadType>(ptr->GetPayloadType()),
            ptr->GetBuffer(),
            ptr->GetPayloadSize(),
            view(static_cast<const Pylon::IImage&>(ptr)),
            ptr->GetOffsetX(),
            ptr->GetOffsetY(),
            ptr->GetPaddingY(),
            ptr->GetBlockID(),
            ptr->GetTimeStamp(),
            ptr->GetID(),
            ptr->GetImageNumber(),
            ptr->GetNumberOfSkippedImages(),
        };
    });
}

PylonStatus pylon_grab_result_error_description(const PylonGrabResult* result,
                                                PylonStringCallback cb, void* ctx) {
    return guard([&] { emit(cb, ctx, result->ptr->GetErrorDescription()); });
}

PylonStatus pylon_grab_result_chunk_node_map(const PylonGrabResult* result, PylonNodeMap** out) {
    return guard([&] { *out = wrap(result->ptr->GetChunkDataNodeMap()); });
}

// ---------------------------------------------------------------------------------------------
// Node map and nodes
// ---------------------------------------------------------------------------------------------

PylonStatus pylon_node_map_node(PylonNodeMap* map, const char* name, PylonNode** out) {
    return guard([&] { *out = wrap(unwrap(map)->GetNode(name)); });
}

PylonStatus pylon_node_map_load(PylonNodeMap* map, const char* features, bool validate) {
    return guard([&] { Pylon::CFeaturePersistence::LoadFromString(features, unwrap(map), validate); });
}

PylonStatus pylon_node_map_save(PylonNodeMap* map, PylonStringCallback cb, void* ctx) {
    return guard([&] {
        Pylon::String_t features;
        Pylon::CFeaturePersistence::SaveToString(features, unwrap(map));
        emit(cb, ctx, features);
    });
}

PylonStatus pylon_node_type(PylonNode* node, PylonNodeType* out) {
    return guard([&] { *out = static_cast<PylonNodeType>(unwrap(node)->GetPrincipalInterfaceType()); });
}

PylonStatus pylon_node_access_mode(PylonNode* node, PylonAccessMode* out) {
    return guard([&] { *out = static_cast<PylonAccessMode>(unwrap(node)->GetAccessMode()); });
}

PylonStatus pylon_node_text(PylonNode* node, PylonNodeText text, PylonStringCallback cb,
                            void* ctx) {
    return guard([&] { emit(cb, ctx, node_text(*unwrap(node), text)); });
}

PylonStatus pylon_node_register_callback(PylonNode* node, PylonNodeCallback cb, void* ctx,
                                         intptr_t* out) {
    return guard([&] {
        *out = GenApi::Register(unwrap(node), NodeCallback{cb, ctx}, GenApi::cbPostOutsideLock);
    });
}

PylonStatus pylon_node_deregister_callback(intptr_t registration) {
    return guard([&] { GenApi::Deregister(registration); });
}

PylonStatus pylon_value_to_string(PylonNode* node, PylonStringCallback cb, void* ctx) {
    return guard([&] { emit(cb, ctx, as<GenApi::IValue>(node).ToString()); });
}

PylonStatus pylon_value_from_string(PylonNode* node, const char* value) {
    return guard([&] { as<GenApi::IValue>(node).FromString(value); });
}

PylonStatus pylon_integer_get(PylonNode* node, int64_t* out) {
    return guard([&] { *out = as<GenApi::IInteger>(node).GetValue(); });
}

PylonStatus pylon_integer_set(PylonNode* node, int64_t value) {
    return guard([&] { as<GenApi::IInteger>(node).SetValue(value); });
}

PylonStatus pylon_integer_range(PylonNode* node, int64_t* min, int64_t* max, int64_t* inc) {
    return guard([&] {
        GenApi::IInteger& integer = as<GenApi::IInteger>(node);
        *min = integer.GetMin();
        *max = integer.GetMax();
        *inc = integer.GetInc();
    });
}

PylonStatus pylon_float_get(PylonNode* node, double* out) {
    return guard([&] { *out = as<GenApi::IFloat>(node).GetValue(); });
}

PylonStatus pylon_float_set(PylonNode* node, double value) {
    return guard([&] { as<GenApi::IFloat>(node).SetValue(value); });
}

PylonStatus pylon_float_range(PylonNode* node, double* min, double* max, double* inc) {
    return guard([&] {
        GenApi::IFloat& number = as<GenApi::IFloat>(node);
        *min = number.GetMin();
        *max = number.GetMax();
        *inc = number.HasInc() ? number.GetInc() : 0.0;
    });
}

PylonStatus pylon_boolean_get(PylonNode* node, bool* out) {
    return guard([&] { *out = as<GenApi::IBoolean>(node).GetValue(); });
}

PylonStatus pylon_boolean_set(PylonNode* node, bool value) {
    return guard([&] { as<GenApi::IBoolean>(node).SetValue(value); });
}

PylonStatus pylon_command_execute(PylonNode* node) {
    return guard([&] { as<GenApi::ICommand>(node).Execute(); });
}

PylonStatus pylon_command_is_done(PylonNode* node, bool* out) {
    return guard([&] { *out = as<GenApi::ICommand>(node).IsDone(); });
}

PylonStatus pylon_enumeration_entries(PylonNode* node, PylonNodeCallback cb, void* ctx) {
    return guard([&] {
        GenApi::NodeList_t entries;
        as<GenApi::IEnumeration>(node).GetEntries(entries);
        for (GenApi::INode* entry : entries) {
            cb(ctx, wrap(entry));
        }
    });
}

PylonStatus pylon_enum_entry_symbolic(PylonNode* node, PylonStringCallback cb, void* ctx) {
    return guard([&] { emit(cb, ctx, as<GenApi::IEnumEntry>(node).GetSymbolic()); });
}

PylonStatus pylon_category_features(PylonNode* node, PylonNodeCallback cb, void* ctx) {
    return guard([&] {
        GenApi::FeatureList_t features;
        as<GenApi::ICategory>(node).GetFeatures(features);
        for (GenApi::IValue* feature : features) {
            cb(ctx, wrap(feature->GetNode()));
        }
    });
}

// ---------------------------------------------------------------------------------------------
// Image and format converter
// ---------------------------------------------------------------------------------------------

PylonStatus pylon_image_create(PylonImage** out) {
    return guard([&] { *out = new PylonImage{}; });
}

void pylon_image_destroy(PylonImage* image) {
    delete image;
}

PylonImageView pylon_image_view(const PylonImage* image) {
    return view(image->image);
}

PylonStatus pylon_converter_create(PylonConverter** out) {
    return guard([&] { *out = new PylonConverter{}; });
}

void pylon_converter_destroy(PylonConverter* converter) {
    delete converter;
}

PylonNodeMap* pylon_converter_node_map(PylonConverter* converter) {
    return wrap(converter->converter.GetNodeMap());
}

PylonStatus pylon_converter_set_output_pixel_type(PylonConverter* converter, uint32_t pixel_type) {
    return guard([&] { converter->converter.OutputPixelFormat.SetValue(to_pixel_type(pixel_type)); });
}

bool pylon_converter_has_destination_format(const PylonConverter* converter,
                                            const PylonImageView* source) {
    return converter->converter.ImageHasDestinationFormat(
        to_pixel_type(source->pixel_type), source->padding_x,
        static_cast<Pylon::EImageOrientation>(source->orientation));
}

PylonStatus pylon_converter_convert(PylonConverter* converter, PylonImage* destination,
                                    const PylonImageView* source) {
    return guard([&] {
        converter->converter.Convert(destination->image, source->buffer, source->size,
                                     to_pixel_type(source->pixel_type), source->width,
                                     source->height, source->padding_x,
                                     static_cast<Pylon::EImageOrientation>(source->orientation));
    });
}

} // extern "C"
