/*
 * pylon_shim.h - C ABI over the Basler pylon C++ SDK (pylon 12).
 *
 * Handles    Owned handles are released with their *_destroy function. Borrowed handles
 *            (PylonNodeMap, PylonNode, image buffers) are never released; each accessor states
 *            how long they stay valid. Handle arguments are non-NULL unless stated otherwise.
 * Errors     Functions returning PylonStatus turn every C++ exception into a status; the
 *            description of the last failure on the calling thread is returned by
 *            pylon_last_error(). Out parameters are meaningful only on PYLON_OK. Functions
 *            returning other types cannot fail.
 * Strings    Input strings are NUL-terminated UTF-8. Output strings are passed to a
 *            PylonStringCallback before the function returns, one call per string; the data is
 *            valid only during the call.
 * Callbacks  Callbacks must not unwind.
 * Enums      Values reported by pylon may lie outside the listed constants.
 * Threads    PylonCamera is synchronized by pylon, but only one thread at a time may wait in
 *            pylon_camera_retrieve_result or pylon_camera_grab_one. PylonGrabResult may be used
 *            from any thread. PylonDeviceInfo, PylonConverter and PylonImage are unsynchronized.
 * Runtime    Every object must be destroyed before the last pylon_terminate().
 */

#ifndef PYLON_SHIM_H
#define PYLON_SHIM_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================================
 * Handles
 * ========================================================================================== */

typedef struct PylonDeviceInfo PylonDeviceInfo; /* owned:    Pylon::CDeviceInfo */
typedef struct PylonCamera PylonCamera;         /* owned:    Pylon::CInstantCamera with its device */
typedef struct PylonGrabResult PylonGrabResult; /* owned:    one reference of Pylon::CGrabResultPtr */
typedef struct PylonConverter PylonConverter;   /* owned:    Pylon::CImageFormatConverter */
typedef struct PylonImage PylonImage;           /* owned:    Pylon::CPylonImage */
typedef struct PylonNodeMap PylonNodeMap;       /* borrowed: GenApi::INodeMap */
typedef struct PylonNode PylonNode;             /* borrowed: GenApi::INode */

/* ============================================================================================
 * Status and callbacks
 * ========================================================================================== */

typedef enum PylonStatus {
    PYLON_OK = 0,
    PYLON_ERROR_GENERIC,          /* GenICam::GenericException */
    PYLON_ERROR_BAD_ALLOC,        /* GenICam::BadAllocException, std::bad_alloc */
    PYLON_ERROR_INVALID_ARGUMENT, /* GenICam::InvalidArgumentException */
    PYLON_ERROR_OUT_OF_RANGE,     /* GenICam::OutOfRangeException */
    PYLON_ERROR_PROPERTY,         /* GenICam::PropertyException */
    PYLON_ERROR_RUNTIME,          /* GenICam::RuntimeException */
    PYLON_ERROR_LOGICAL,          /* GenICam::LogicalErrorException */
    PYLON_ERROR_ACCESS,           /* GenICam::AccessException */
    PYLON_ERROR_TIMEOUT,          /* GenICam::TimeoutException */
    PYLON_ERROR_DYNAMIC_CAST,     /* GenICam::DynamicCastException, node of another type */
    PYLON_ERROR_UNKNOWN           /* any other C++ exception */
} PylonStatus;

/* Receives one string; data is not NUL-terminated by contract. */
typedef void (*PylonStringCallback)(void* ctx, const char* data, size_t len);

/* Receives one borrowed node. */
typedef void (*PylonNodeCallback)(void* ctx, PylonNode* node);

/* Receives one device info; the callee owns it. */
typedef void (*PylonDeviceInfoCallback)(void* ctx, PylonDeviceInfo* info);

/* Timeout that waits forever. */
#define PYLON_INFINITE 0xFFFFFFFFu

/* Description of the last failure on the calling thread, valid until the next failure on it. */
const char* pylon_last_error(void);

/* ============================================================================================
 * Runtime
 * ========================================================================================== */

typedef struct PylonVersion {
    uint32_t major;
    uint32_t minor;
    uint32_t subminor;
    uint32_t build;
} PylonVersion;

/* Pylon::PylonInitialize. Reference counted: every call needs one pylon_terminate(). */
PylonStatus pylon_initialize(void);

/* Pylon::PylonTerminate. */
PylonStatus pylon_terminate(void);

/* Pylon::GetPylonVersion. */
PylonVersion pylon_version(void);

/* ============================================================================================
 * Pixel types
 *
 * Values of Pylon::EPixelType. A name is the pylon name with word boundaries turned into
 * underscores; the PFNC suffixes p, s and f stay attached to the bit depth.
 * ========================================================================================== */

#define PYLON_PIXEL_TYPE_UNDEFINED                     0xFFFFFFFFu

#define PYLON_PIXEL_TYPE_MONO1_PACKED                  0x8101000Cu
#define PYLON_PIXEL_TYPE_MONO2_PACKED                  0x8102000Du
#define PYLON_PIXEL_TYPE_MONO4_PACKED                  0x8104000Eu
#define PYLON_PIXEL_TYPE_MONO8                         0x01080001u
#define PYLON_PIXEL_TYPE_MONO8_SIGNED                  0x01080002u
#define PYLON_PIXEL_TYPE_MONO10                        0x01100003u
#define PYLON_PIXEL_TYPE_MONO10_PACKED                 0x010C0004u
#define PYLON_PIXEL_TYPE_MONO10P                       0x010A0046u
#define PYLON_PIXEL_TYPE_MONO12                        0x01100005u
#define PYLON_PIXEL_TYPE_MONO12_PACKED                 0x010C0006u
#define PYLON_PIXEL_TYPE_MONO12P                       0x010C0047u
#define PYLON_PIXEL_TYPE_MONO16                        0x01100007u

#define PYLON_PIXEL_TYPE_BAYER_GR8                     0x01080008u
#define PYLON_PIXEL_TYPE_BAYER_RG8                     0x01080009u
#define PYLON_PIXEL_TYPE_BAYER_GB8                     0x0108000Au
#define PYLON_PIXEL_TYPE_BAYER_BG8                     0x0108000Bu
#define PYLON_PIXEL_TYPE_BAYER_GR10                    0x0110000Cu
#define PYLON_PIXEL_TYPE_BAYER_RG10                    0x0110000Du
#define PYLON_PIXEL_TYPE_BAYER_GB10                    0x0110000Eu
#define PYLON_PIXEL_TYPE_BAYER_BG10                    0x0110000Fu
#define PYLON_PIXEL_TYPE_BAYER_GR12                    0x01100010u
#define PYLON_PIXEL_TYPE_BAYER_RG12                    0x01100011u
#define PYLON_PIXEL_TYPE_BAYER_GB12                    0x01100012u
#define PYLON_PIXEL_TYPE_BAYER_BG12                    0x01100013u
#define PYLON_PIXEL_TYPE_BAYER_GR12_PACKED             0x010C002Au
#define PYLON_PIXEL_TYPE_BAYER_RG12_PACKED             0x010C002Bu
#define PYLON_PIXEL_TYPE_BAYER_GB12_PACKED             0x010C002Cu
#define PYLON_PIXEL_TYPE_BAYER_BG12_PACKED             0x010C002Du
#define PYLON_PIXEL_TYPE_BAYER_GR10P                   0x010A0056u
#define PYLON_PIXEL_TYPE_BAYER_RG10P                   0x010A0058u
#define PYLON_PIXEL_TYPE_BAYER_GB10P                   0x010A0054u
#define PYLON_PIXEL_TYPE_BAYER_BG10P                   0x010A0052u
#define PYLON_PIXEL_TYPE_BAYER_GR12P                   0x010C0057u
#define PYLON_PIXEL_TYPE_BAYER_RG12P                   0x010C0059u
#define PYLON_PIXEL_TYPE_BAYER_GB12P                   0x010C0055u
#define PYLON_PIXEL_TYPE_BAYER_BG12P                   0x010C0053u
#define PYLON_PIXEL_TYPE_BAYER_GR16                    0x0110002Eu
#define PYLON_PIXEL_TYPE_BAYER_RG16                    0x0110002Fu
#define PYLON_PIXEL_TYPE_BAYER_GB16                    0x01100030u
#define PYLON_PIXEL_TYPE_BAYER_BG16                    0x01100031u

#define PYLON_PIXEL_TYPE_RGB8_PACKED                   0x02180014u
#define PYLON_PIXEL_TYPE_BGR8_PACKED                   0x02180015u
#define PYLON_PIXEL_TYPE_RGBA8_PACKED                  0x02200016u
#define PYLON_PIXEL_TYPE_BGRA8_PACKED                  0x02200017u
#define PYLON_PIXEL_TYPE_RGB10_PACKED                  0x02300018u
#define PYLON_PIXEL_TYPE_BGR10_PACKED                  0x02300019u
#define PYLON_PIXEL_TYPE_RGB12_PACKED                  0x0230001Au
#define PYLON_PIXEL_TYPE_BGR12_PACKED                  0x0230001Bu
#define PYLON_PIXEL_TYPE_RGB16_PACKED                  0x02300033u
#define PYLON_PIXEL_TYPE_BGR10V1_PACKED                0x0220001Cu
#define PYLON_PIXEL_TYPE_BGR10V2_PACKED                0x0220001Du
#define PYLON_PIXEL_TYPE_RGB12V1_PACKED                0x02240034u
#define PYLON_PIXEL_TYPE_RGB8_PLANAR                   0x02180021u
#define PYLON_PIXEL_TYPE_RGB10_PLANAR                  0x02300022u
#define PYLON_PIXEL_TYPE_RGB12_PLANAR                  0x02300023u
#define PYLON_PIXEL_TYPE_RGB16_PLANAR                  0x02300024u

#define PYLON_PIXEL_TYPE_YUV411_PACKED                 0x020C001Eu
#define PYLON_PIXEL_TYPE_YUV422_PACKED                 0x0210001Fu
#define PYLON_PIXEL_TYPE_YUV444_PACKED                 0x02180020u
#define PYLON_PIXEL_TYPE_YUV422_YUYV_PACKED            0x02100032u
#define PYLON_PIXEL_TYPE_YUV444_PLANAR                 0x82180044u
#define PYLON_PIXEL_TYPE_YUV422_PLANAR                 0x82100042u
#define PYLON_PIXEL_TYPE_YUV420_PLANAR                 0x820C0040u
#define PYLON_PIXEL_TYPE_YCBCR420_8_YY_CBCR_SEMIPLANAR 0x020C0112u
#define PYLON_PIXEL_TYPE_YCBCR422_8_YY_CBCR_SEMIPLANAR 0x02100113u

#define PYLON_PIXEL_TYPE_BICOLOR_RGBG8                 0x021000A5u
#define PYLON_PIXEL_TYPE_BICOLOR_BGRG8                 0x021000A6u
#define PYLON_PIXEL_TYPE_BICOLOR_RGBG10                0x022000A7u
#define PYLON_PIXEL_TYPE_BICOLOR_RGBG10P               0x021400A8u
#define PYLON_PIXEL_TYPE_BICOLOR_BGRG10                0x022000A9u
#define PYLON_PIXEL_TYPE_BICOLOR_BGRG10P               0x021400AAu
#define PYLON_PIXEL_TYPE_BICOLOR_RGBG12                0x022000ABu
#define PYLON_PIXEL_TYPE_BICOLOR_RGBG12P               0x021800ACu
#define PYLON_PIXEL_TYPE_BICOLOR_BGRG12                0x022000ADu
#define PYLON_PIXEL_TYPE_BICOLOR_BGRG12P               0x021800AEu

#define PYLON_PIXEL_TYPE_DOUBLE                        0x81400100u
#define PYLON_PIXEL_TYPE_CONFIDENCE8                   0x010800C6u
#define PYLON_PIXEL_TYPE_CONFIDENCE16                  0x011000C7u
#define PYLON_PIXEL_TYPE_COORD3D_C8                    0x010800B1u
#define PYLON_PIXEL_TYPE_COORD3D_C16                   0x011000B8u
#define PYLON_PIXEL_TYPE_COORD3D_ABC32F                0x026000C0u
#define PYLON_PIXEL_TYPE_ERROR8                        0x81080001u
#define PYLON_PIXEL_TYPE_DATA8                         0x01080116u
#define PYLON_PIXEL_TYPE_DATA8S                        0x01080117u
#define PYLON_PIXEL_TYPE_DATA16                        0x01100118u
#define PYLON_PIXEL_TYPE_DATA16S                       0x01100119u
#define PYLON_PIXEL_TYPE_DATA32                        0x0120011Au
#define PYLON_PIXEL_TYPE_DATA32S                       0x0120011Bu
#define PYLON_PIXEL_TYPE_DATA32F                       0x0120011Cu
#define PYLON_PIXEL_TYPE_DATA64                        0x0140011Du
#define PYLON_PIXEL_TYPE_DATA64S                       0x0140011Eu
#define PYLON_PIXEL_TYPE_DATA64F                       0x0140011Fu

/* Properties that pylon leaves undefined for a pixel type are 0. */
typedef struct PylonPixelTypeInfo {
    const char* name;           /* CPixelTypeMapper::GetNameByPixelType (SFNC 2.0), static storage,
                                   empty for the planar YUV types */
    uint32_t bits_per_pixel;    /* Pylon::BitPerPixel */
    uint32_t bit_depth;         /* Pylon::BitDepth */
    uint32_t samples_per_pixel; /* Pylon::SamplesPerPixel */
    uint32_t plane_count;       /* Pylon::PlaneCount */
    bool is_mono;               /* Pylon::IsMonoImage; Pylon::IsColorImage is its complement */
    bool is_bayer;              /* Pylon::IsBayer */
    bool is_packed;             /* Pylon::IsPacked */
    bool has_alpha;             /* Pylon::HasAlpha */
} PylonPixelTypeInfo;

/* Properties of a pixel type. Fails for values that are not pixel types, e.g. UNDEFINED. */
PylonStatus pylon_pixel_type_info(uint32_t pixel_type, PylonPixelTypeInfo* out);

/* CPixelTypeMapper::GetPylonPixelTypeByName, e.g. the symbolic of the camera PixelFormat.
 * Returns PYLON_PIXEL_TYPE_UNDEFINED for unknown names. */
uint32_t pylon_pixel_type_from_name(const char* name);

/* Pylon::ComputeStride. Writes 0 where pylon cannot compute a byte stride: rows that are not
 * byte aligned and have no padding, planar YUV422 and YUV420. */
PylonStatus pylon_pixel_type_stride(uint32_t pixel_type, uint32_t width, size_t padding_x,
                                    size_t* out);

/* ============================================================================================
 * Image layout
 * ========================================================================================== */

typedef enum PylonImageOrientation { /* Pylon::EImageOrientation */
    PYLON_IMAGE_ORIENTATION_TOP_DOWN,
    PYLON_IMAGE_ORIENTATION_BOTTOM_UP
} PylonImageOrientation;

/* Image memory described by its layout; the buffer belongs to the source of the view. */
typedef struct PylonImageView {
    const void* buffer;                /* NULL for an invalid image */
    size_t size;                       /* image bytes */
    uint32_t pixel_type;               /* PYLON_PIXEL_TYPE_* */
    uint32_t width;
    uint32_t height;
    size_t padding_x;                  /* extra bytes at the end of each row */
    PylonImageOrientation orientation;
} PylonImageView;

/* ============================================================================================
 * Device info
 *
 * Pylon::CDeviceInfo is a property table; keys are the names of Pylon::Key, e.g. SerialNumber,
 * ModelName, UserDefinedName, FullName, DeviceClass.
 * ========================================================================================== */

/* Empty device info, used as enumeration filter or to select a device. */
PylonStatus pylon_device_info_create(PylonDeviceInfo** out);

void pylon_device_info_destroy(PylonDeviceInfo* info);

/* CDeviceInfo::GetPropertyValue. cb runs only if the property exists. */
PylonStatus pylon_device_info_get(const PylonDeviceInfo* info, const char* key,
                                  PylonStringCallback cb, void* ctx);

/* CDeviceInfo::SetPropertyValue. */
PylonStatus pylon_device_info_set(PylonDeviceInfo* info, const char* key, const char* value);

/* CDeviceInfo::GetPropertyNames. */
PylonStatus pylon_device_info_keys(const PylonDeviceInfo* info, PylonStringCallback cb,
                                   void* ctx);

/* CTlFactory::EnumerateDevices. Reports the devices matching any filter, or all devices if
 * filter_count is 0 (filters may then be NULL). */
PylonStatus pylon_enumerate_devices(const PylonDeviceInfo* const* filters, size_t filter_count,
                                    PylonDeviceInfoCallback cb, void* ctx);

/* ============================================================================================
 * Camera
 *
 * A camera holds its device from pylon_camera_create until pylon_camera_destroy; after a device
 * removal the camera is destroyed and created again.
 * ========================================================================================== */

typedef enum PylonNodeMapKind {
    PYLON_NODE_MAP_DEVICE,          /* GetNodeMap: camera features */
    PYLON_NODE_MAP_TRANSPORT_LAYER, /* GetTLNodeMap */
    PYLON_NODE_MAP_STREAM_GRABBER,  /* GetStreamGrabberNodeMap, requires an open camera */
    PYLON_NODE_MAP_EVENT_GRABBER,   /* GetEventGrabberNodeMap, requires an open camera */
    PYLON_NODE_MAP_INSTANT_CAMERA   /* GetInstantCameraNodeMap: MaxNumBuffer, GrabCameraEvents... */
} PylonNodeMapKind;

typedef enum PylonConfiguration {
    PYLON_CONFIGURATION_NONE,
    PYLON_CONFIGURATION_ACQUIRE_CONTINUOUS,   /* CAcquireContinuousConfiguration, pylon default */
    PYLON_CONFIGURATION_ACQUIRE_SINGLE_FRAME, /* CAcquireSingleFrameConfiguration */
    PYLON_CONFIGURATION_SOFTWARE_TRIGGER      /* CSoftwareTriggerConfiguration */
} PylonConfiguration;

typedef enum PylonGrabStrategy { /* Pylon::EGrabStrategy */
    PYLON_GRAB_STRATEGY_ONE_BY_ONE,
    PYLON_GRAB_STRATEGY_LATEST_IMAGE_ONLY,
    PYLON_GRAB_STRATEGY_LATEST_IMAGES,
    PYLON_GRAB_STRATEGY_UPCOMING_IMAGE
} PylonGrabStrategy;

/* CTlFactory::CreateDevice(info), or CreateFirstDevice() if info is NULL, attached to a new
 * CInstantCamera. The camera is closed. */
PylonStatus pylon_camera_create(const PylonDeviceInfo* info, PylonCamera** out);

/* Stops grabbing, closes the camera and destroys its device. */
void pylon_camera_destroy(PylonCamera* camera);

/* Copy of CInstantCamera::GetDeviceInfo. */
PylonStatus pylon_camera_device_info(const PylonCamera* camera, PylonDeviceInfo** out);

/* CInstantCamera::Open. Applies the configuration set by pylon_camera_set_configuration. */
PylonStatus pylon_camera_open(PylonCamera* camera);

/* CInstantCamera::Close. Stops grabbing first. */
void pylon_camera_close(PylonCamera* camera);

/* CInstantCamera::IsOpen. */
bool pylon_camera_is_open(const PylonCamera* camera);

/* CInstantCamera::IsCameraDeviceRemoved. Detection requires an open camera. */
bool pylon_camera_is_device_removed(const PylonCamera* camera);

/* Node map of the camera. DEVICE, STREAM_GRABBER and EVENT_GRABBER stay valid until
 * pylon_camera_close or pylon_camera_destroy; TRANSPORT_LAYER and INSTANT_CAMERA until
 * pylon_camera_destroy. */
PylonStatus pylon_camera_node_map(PylonCamera* camera, PylonNodeMapKind kind, PylonNodeMap** out);

/* CInstantCamera::RegisterConfiguration with RegistrationMode_ReplaceAll. Takes effect when the
 * camera is opened next. */
PylonStatus pylon_camera_set_configuration(PylonCamera* camera, PylonConfiguration configuration);

/* CInstantCamera::Open, then StartGrabbing with GrabLoop_ProvidedByUser; the camera stays open
 * when grabbing stops. max_images 0 grabs until pylon_camera_stop_grabbing, otherwise grabbing
 * stops after max_images results have been retrieved. */
PylonStatus pylon_camera_start_grabbing(PylonCamera* camera, PylonGrabStrategy strategy,
                                        size_t max_images);

/* CInstantCamera::StopGrabbing. A thread waiting in pylon_camera_retrieve_result returns at once
 * with NULL. */
void pylon_camera_stop_grabbing(PylonCamera* camera);

/* CInstantCamera::IsGrabbing. */
bool pylon_camera_is_grabbing(const PylonCamera* camera);

/* CInstantCamera::RetrieveResult with TimeoutHandling_Return. Writes NULL if the timeout
 * expired or the camera is not grabbing. Node callbacks of camera events run inside this call. */
PylonStatus pylon_camera_retrieve_result(PylonCamera* camera, uint32_t timeout_ms,
                                         PylonGrabResult** out);

/* CInstantCamera::Open, then GrabOne with TimeoutHandling_Return: pylon_camera_start_grabbing
 * with max_images 1, one retrieve and pylon_camera_stop_grabbing, also on timeout. Writes NULL
 * if the timeout expired. */
PylonStatus pylon_camera_grab_one(PylonCamera* camera, uint32_t timeout_ms,
                                  PylonGrabResult** out);

/* CInstantCamera::CanWaitForFrameTriggerReady. */
PylonStatus pylon_camera_can_wait_for_frame_trigger_ready(const PylonCamera* camera, bool* out);

/* CInstantCamera::WaitForFrameTriggerReady with TimeoutHandling_Return. Writes false if the
 * timeout expired. */
PylonStatus pylon_camera_wait_for_frame_trigger_ready(PylonCamera* camera, uint32_t timeout_ms,
                                                      bool* out);

/* CInstantCamera::ExecuteSoftwareTrigger. */
PylonStatus pylon_camera_execute_software_trigger(PylonCamera* camera);

/* ============================================================================================
 * Grab result
 *
 * A grab result may outlive its camera. Its buffers stay valid until it is destroyed; holding
 * results keeps their buffers out of the grab queue.
 * ========================================================================================== */

typedef enum PylonPayloadType { /* Pylon::EPayloadType */
    PYLON_PAYLOAD_TYPE_UNDEFINED = -1,
    PYLON_PAYLOAD_TYPE_IMAGE,
    PYLON_PAYLOAD_TYPE_RAW_DATA,
    PYLON_PAYLOAD_TYPE_FILE,
    PYLON_PAYLOAD_TYPE_CHUNK_DATA,
    PYLON_PAYLOAD_TYPE_GEN_DC,
    PYLON_PAYLOAD_TYPE_DEVICE_SPECIFIC = 0x8000
} PylonPayloadType;

typedef struct PylonGrabResultInfo {
    bool succeeded;                /* GrabSucceeded */
    uint32_t error_code;           /* GetErrorCode */
    PylonPayloadType payload_type; /* GetPayloadType */
    const void* payload;           /* GetBuffer: whole payload including chunk data */
    size_t payload_size;           /* GetPayloadSize */
    PylonImageView image;          /* the result as Pylon::IImage, invalid if the grab failed;
                                      the first image component of a GenDC payload */
    uint32_t offset_x;             /* GetOffsetX */
    uint32_t offset_y;             /* GetOffsetY */
    uint32_t padding_y;            /* GetPaddingY */
    uint64_t block_id;             /* GetBlockID */
    uint64_t timestamp;            /* GetTimeStamp */
    int64_t id;                    /* GetID */
    int64_t image_number;          /* GetImageNumber */
    int64_t skipped_images;        /* GetNumberOfSkippedImages */
} PylonGrabResultInfo;

void pylon_grab_result_destroy(PylonGrabResult* result);

PylonStatus pylon_grab_result_info(const PylonGrabResult* result, PylonGrabResultInfo* out);

/* CGrabResultData::GetErrorDescription. */
PylonStatus pylon_grab_result_error_description(const PylonGrabResult* result,
                                                PylonStringCallback cb, void* ctx);

/* CGrabResultData::GetChunkDataNodeMap, valid until the result is destroyed. The node map is
 * empty if chunks are disabled. */
PylonStatus pylon_grab_result_chunk_node_map(const PylonGrabResult* result, PylonNodeMap** out);

/* ============================================================================================
 * Node map and nodes
 *
 * A node stays valid as long as its node map. Typed accessors fail with
 * PYLON_ERROR_DYNAMIC_CAST if the node does not implement the GenApi interface of their prefix.
 * ========================================================================================== */

typedef enum PylonNodeType { /* GenApi::EInterfaceType */
    PYLON_NODE_TYPE_VALUE,
    PYLON_NODE_TYPE_BASE,
    PYLON_NODE_TYPE_INTEGER,
    PYLON_NODE_TYPE_BOOLEAN,
    PYLON_NODE_TYPE_COMMAND,
    PYLON_NODE_TYPE_FLOAT,
    PYLON_NODE_TYPE_STRING,
    PYLON_NODE_TYPE_REGISTER,
    PYLON_NODE_TYPE_CATEGORY,
    PYLON_NODE_TYPE_ENUMERATION,
    PYLON_NODE_TYPE_ENUM_ENTRY,
    PYLON_NODE_TYPE_PORT
} PylonNodeType;

typedef enum PylonAccessMode { /* GenApi::EAccessMode */
    PYLON_ACCESS_MODE_NI, /* not implemented */
    PYLON_ACCESS_MODE_NA, /* not available */
    PYLON_ACCESS_MODE_WO,
    PYLON_ACCESS_MODE_RO,
    PYLON_ACCESS_MODE_RW
} PylonAccessMode;

typedef enum PylonNodeText {
    PYLON_NODE_TEXT_NAME,
    PYLON_NODE_TEXT_DISPLAY_NAME,
    PYLON_NODE_TEXT_TOOL_TIP,
    PYLON_NODE_TEXT_DESCRIPTION
} PylonNodeText;

/* INodeMap::GetNode. Writes NULL if the node map has no node of that name. */
PylonStatus pylon_node_map_node(PylonNodeMap* map, const char* name, PylonNode** out);

/* CFeaturePersistence::LoadFromString. */
PylonStatus pylon_node_map_load(PylonNodeMap* map, const char* features, bool validate);

/* CFeaturePersistence::SaveToString. */
PylonStatus pylon_node_map_save(PylonNodeMap* map, PylonStringCallback cb, void* ctx);

/* INode::GetPrincipalInterfaceType. */
PylonStatus pylon_node_type(PylonNode* node, PylonNodeType* out);

/* IBase::GetAccessMode. */
PylonStatus pylon_node_access_mode(PylonNode* node, PylonAccessMode* out);

/* INode::GetName, GetDisplayName, GetToolTip or GetDescription. */
PylonStatus pylon_node_text(PylonNode* node, PylonNodeText text, PylonStringCallback cb,
                            void* ctx);

/* GenApi::Register. cb runs on the thread that changes the node, outside the node map
 * lock, until pylon_node_deregister_callback or the end of the node map. Camera events update
 * their data nodes (e.g. EventExposureEndData) inside pylon_camera_retrieve_result once
 * GrabCameraEvents is enabled in the INSTANT_CAMERA node map before pylon_camera_open. */
PylonStatus pylon_node_register_callback(PylonNode* node, PylonNodeCallback cb, void* ctx,
                                         intptr_t* out);

/* GenApi::Deregister. Requires the node map of the registration to be valid. */
PylonStatus pylon_node_deregister_callback(intptr_t registration);

/* IValue::ToString; the symbolic of the current entry of an enumeration, the value of a string,
 * the numeric value of an enumeration entry. */
PylonStatus pylon_value_to_string(PylonNode* node, PylonStringCallback cb, void* ctx);

/* IValue::FromString; selects an enumeration entry by its symbolic, sets a string. */
PylonStatus pylon_value_from_string(PylonNode* node, const char* value);

/* IInteger. */
PylonStatus pylon_integer_get(PylonNode* node, int64_t* out);
PylonStatus pylon_integer_set(PylonNode* node, int64_t value);
PylonStatus pylon_integer_range(PylonNode* node, int64_t* min, int64_t* max, int64_t* inc);

/* IFloat. inc is 0 if the node has no increment. */
PylonStatus pylon_float_get(PylonNode* node, double* out);
PylonStatus pylon_float_set(PylonNode* node, double value);
PylonStatus pylon_float_range(PylonNode* node, double* min, double* max, double* inc);

/* IBoolean. */
PylonStatus pylon_boolean_get(PylonNode* node, bool* out);
PylonStatus pylon_boolean_set(PylonNode* node, bool value);

/* ICommand. */
PylonStatus pylon_command_execute(PylonNode* node);
PylonStatus pylon_command_is_done(PylonNode* node, bool* out);

/* IEnumeration::GetEntries. Entries are nodes; their access mode tells whether they can be
 * selected. */
PylonStatus pylon_enumeration_entries(PylonNode* node, PylonNodeCallback cb, void* ctx);

/* IEnumEntry::GetSymbolic, the value pylon_value_from_string selects the entry with. */
PylonStatus pylon_enum_entry_symbolic(PylonNode* node, PylonStringCallback cb, void* ctx);

/* ICategory::GetFeatures. */
PylonStatus pylon_category_features(PylonNode* node, PylonNodeCallback cb, void* ctx);

/* ============================================================================================
 * Image and format converter
 * ========================================================================================== */

/* Empty Pylon::CPylonImage, used as conversion target; its buffer is reused while large
 * enough. */
PylonStatus pylon_image_create(PylonImage** out);

void pylon_image_destroy(PylonImage* image);

/* Layout of the image; the buffer stays valid until the next conversion into the image or its
 * destruction. */
PylonImageView pylon_image_view(const PylonImage* image);

PylonStatus pylon_converter_create(PylonConverter** out);

void pylon_converter_destroy(PylonConverter* converter);

/* CImageFormatConverter::GetNodeMap: Gamma, MonoConversionMethod, OutputBitAlignment,
 * OutputOrientation, OutputPaddingX, InconvertibleEdgeHandling, MaxNumThreads, ...
 * Valid until the converter is destroyed. */
PylonNodeMap* pylon_converter_node_map(PylonConverter* converter);

/* CImageFormatConverter::OutputPixelFormat. */
PylonStatus pylon_converter_set_output_pixel_type(PylonConverter* converter, uint32_t pixel_type);

/* CImageFormatConverter::ImageHasDestinationFormat: true if converting the source under the
 * current settings would only copy it, so the source can be used as it is. */
bool pylon_converter_has_destination_format(const PylonConverter* converter,
                                            const PylonImageView* source);

/* CImageFormatConverter::Convert of the source into the destination image. */
PylonStatus pylon_converter_convert(PylonConverter* converter, PylonImage* destination,
                                    const PylonImageView* source);

#ifdef __cplusplus
}
#endif

#endif /* PYLON_SHIM_H */
