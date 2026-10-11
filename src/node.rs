//! GenApi node maps and nodes.

use crate::{Result, c_string, check, optional_handle, out, push_handle, string, sys};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Borrowed `GenApi::INodeMap`, valid while its camera, grab result or converter is borrowed.
#[derive(Clone, Copy)]
pub struct NodeMap<'a> {
    pub(crate) raw: NonNull<sys::PylonNodeMap>,
    _host: PhantomData<&'a ()>,
}

impl<'a> NodeMap<'a> {
    /// # Safety
    /// `raw` is a node map that stays valid for `'a`.
    pub(crate) unsafe fn from_raw(raw: NonNull<sys::PylonNodeMap>) -> Self {
        NodeMap { raw, _host: PhantomData }
    }

    /// `INodeMap::GetNode`; None if the map has no node of that name.
    pub fn node(self, name: &str) -> Result<Option<Node<'a>>> {
        let name = c_string(name)?;
        // SAFETY: the map is valid for 'a; name is a C string; the out parameter is a valid handle
        // pointer.
        let node = optional_handle(|node| unsafe {
            sys::pylon_node_map_node(self.raw.as_ptr(), name.as_ptr(), node)
        })?;
        // SAFETY: a node stays valid as long as its node map (pylon_shim.h), so for 'a.
        Ok(node.map(|node| unsafe { Node::from_raw(node) }))
    }

    /// `CFeaturePersistence::LoadFromString`. Values are written without range checks; validation
    /// reports values outside the node ranges afterwards, when they are already written. Such
    /// values can break the host, e.g. a converter MaxNumThreads below 1 makes pylon create threads
    /// without limit, so write untrusted values with the typed setters. On failure some features
    /// may already be written.
    pub fn load(self, features: &str, validate: bool) -> Result<()> {
        let features = c_string(features)?;
        // SAFETY: the map is valid for 'a; features is a C string.
        check(unsafe { sys::pylon_node_map_load(self.raw.as_ptr(), features.as_ptr(), validate) })
    }

    /// `CFeaturePersistence::SaveToString`; the features are read one by one, not atomically.
    pub fn save(self) -> Result<String> {
        // SAFETY: the map is valid for 'a; cb and ctx come from `string`.
        string(|cb, ctx| unsafe { sys::pylon_node_map_save(self.raw.as_ptr(), cb, ctx) })
    }
}

/// Borrowed `GenApi::INode`, valid as long as its node map. The typed accessors fail with
/// [`ErrorKind::DynamicCast`](crate::ErrorKind::DynamicCast) if the node does not implement the
/// GenApi interface they use.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    pub(crate) raw: NonNull<sys::PylonNode>,
    _map: PhantomData<&'a ()>,
}

impl<'a> Node<'a> {
    /// # Safety
    /// `raw` is a node that stays valid for `'a`.
    pub(crate) unsafe fn from_raw(raw: NonNull<sys::PylonNode>) -> Self {
        Node { raw, _map: PhantomData }
    }

    /// `INode::GetPrincipalInterfaceType`.
    pub fn node_type(self) -> Result<NodeType> {
        let mut node_type = sys::PylonNodeType(0);
        // SAFETY: the node is valid for 'a; node_type is a valid out parameter.
        check(unsafe { sys::pylon_node_type(self.raw.as_ptr(), &mut node_type) })?;
        Ok(NodeType(node_type))
    }

    /// `IBase::GetAccessMode`.
    pub fn access_mode(self) -> Result<AccessMode> {
        let mut mode = sys::PylonAccessMode(0);
        // SAFETY: the node is valid for 'a; mode is a valid out parameter.
        check(unsafe { sys::pylon_node_access_mode(self.raw.as_ptr(), &mut mode) })?;
        Ok(AccessMode(mode))
    }

    pub fn name(self) -> Result<String> {
        self.text(sys::PylonNodeText::PYLON_NODE_TEXT_NAME)
    }

    pub fn display_name(self) -> Result<String> {
        self.text(sys::PylonNodeText::PYLON_NODE_TEXT_DISPLAY_NAME)
    }

    pub fn tool_tip(self) -> Result<String> {
        self.text(sys::PylonNodeText::PYLON_NODE_TEXT_TOOL_TIP)
    }

    pub fn description(self) -> Result<String> {
        self.text(sys::PylonNodeText::PYLON_NODE_TEXT_DESCRIPTION)
    }

    fn text(self, text: sys::PylonNodeText) -> Result<String> {
        // SAFETY: the node is valid for 'a; cb and ctx come from `string`.
        string(|cb, ctx| unsafe { sys::pylon_node_text(self.raw.as_ptr(), text, cb, ctx) })
    }

    /// `IValue::ToString`: the symbolic of the current entry of an enumeration, the value of a
    /// string or of a number. For an enumeration entry it is the numeric value; see
    /// [`symbolic`](Self::symbolic).
    pub fn value(self) -> Result<String> {
        // SAFETY: the node is valid for 'a; cb and ctx come from `string`.
        string(|cb, ctx| unsafe { sys::pylon_value_to_string(self.raw.as_ptr(), cb, ctx) })
    }

    /// `IValue::FromString`: selects an enumeration entry by its symbolic, sets a string or a
    /// number.
    pub fn set_value(self, value: &str) -> Result<()> {
        let value = c_string(value)?;
        // SAFETY: the node is valid for 'a; value is a C string.
        check(unsafe { sys::pylon_value_from_string(self.raw.as_ptr(), value.as_ptr()) })
    }

    pub fn integer(self) -> Result<i64> {
        // SAFETY: the node is valid for 'a; the out parameter is a valid i64.
        out(|value| unsafe { sys::pylon_integer_get(self.raw.as_ptr(), value) })
    }

    pub fn set_integer(self, value: i64) -> Result<()> {
        // SAFETY: the node is valid for 'a.
        check(unsafe { sys::pylon_integer_set(self.raw.as_ptr(), value) })
    }

    /// (min, max, inc) of an integer.
    pub fn integer_range(self) -> Result<(i64, i64, i64)> {
        // SAFETY: the node is valid for 'a; min, max and inc are valid out parameters.
        out(|(min, max, inc)| unsafe { sys::pylon_integer_range(self.raw.as_ptr(), min, max, inc) })
    }

    pub fn float(self) -> Result<f64> {
        // SAFETY: the node is valid for 'a; the out parameter is a valid f64.
        out(|value| unsafe { sys::pylon_float_get(self.raw.as_ptr(), value) })
    }

    pub fn set_float(self, value: f64) -> Result<()> {
        // SAFETY: the node is valid for 'a.
        check(unsafe { sys::pylon_float_set(self.raw.as_ptr(), value) })
    }

    /// (min, max, inc) of a float; inc is None if the node has no increment.
    pub fn float_range(self) -> Result<(f64, f64, Option<f64>)> {
        // SAFETY: the node is valid for 'a; min, max and inc are valid out parameters.
        let (min, max, inc): (f64, f64, f64) = out(|(min, max, inc)| unsafe {
            sys::pylon_float_range(self.raw.as_ptr(), min, max, inc)
        })?;
        Ok((min, max, (inc != 0.0).then_some(inc)))
    }

    pub fn boolean(self) -> Result<bool> {
        // SAFETY: the node is valid for 'a; the out parameter is a valid bool.
        out(|value| unsafe { sys::pylon_boolean_get(self.raw.as_ptr(), value) })
    }

    pub fn set_boolean(self, value: bool) -> Result<()> {
        // SAFETY: the node is valid for 'a.
        check(unsafe { sys::pylon_boolean_set(self.raw.as_ptr(), value) })
    }

    /// `ICommand::Execute`.
    pub fn execute(self) -> Result<()> {
        // SAFETY: the node is valid for 'a.
        check(unsafe { sys::pylon_command_execute(self.raw.as_ptr()) })
    }

    /// `ICommand::IsDone`.
    pub fn is_done(self) -> Result<bool> {
        // SAFETY: the node is valid for 'a; the out parameter is a valid bool.
        out(|done| unsafe { sys::pylon_command_is_done(self.raw.as_ptr(), done) })
    }

    /// Entries of an enumeration; their access mode tells whether they can be selected.
    pub fn entries(self) -> Result<Vec<Node<'a>>> {
        // SAFETY: the node is valid for 'a; cb and ctx come from `nodes`.
        nodes(|cb, ctx| unsafe { sys::pylon_enumeration_entries(self.raw.as_ptr(), cb, ctx) })
    }

    /// `IEnumEntry::GetSymbolic`, the value [`set_value`](Self::set_value) selects the entry with.
    pub fn symbolic(self) -> Result<String> {
        // SAFETY: the node is valid for 'a; cb and ctx come from `string`.
        string(|cb, ctx| unsafe { sys::pylon_enum_entry_symbolic(self.raw.as_ptr(), cb, ctx) })
    }

    /// Features of a category.
    pub fn features(self) -> Result<Vec<Node<'a>>> {
        // SAFETY: the node is valid for 'a; cb and ctx come from `nodes`.
        nodes(|cb, ctx| unsafe { sys::pylon_category_features(self.raw.as_ptr(), cb, ctx) })
    }
}

/// Nodes a shim function passes to its node callback, all of the node map of `'a`.
fn nodes<'a>(
    call: impl FnOnce(sys::PylonNodeCallback, *mut c_void) -> sys::PylonStatus,
) -> Result<Vec<Node<'a>>> {
    let mut found = Vec::<NonNull<sys::PylonNode>>::new();
    check(call(Some(push_handle), (&raw mut found).cast()))?;
    // SAFETY: entries and features belong to the node map of the queried node, so they stay valid
    // for 'a.
    Ok(found.into_iter().map(|node| unsafe { Node::from_raw(node) }).collect())
}

/// `GenApi::EInterfaceType`; pylon may report values outside the constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeType(sys::PylonNodeType);

impl NodeType {
    pub const VALUE: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_VALUE);
    pub const BASE: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_BASE);
    pub const INTEGER: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_INTEGER);
    pub const BOOLEAN: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_BOOLEAN);
    pub const COMMAND: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_COMMAND);
    pub const FLOAT: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_FLOAT);
    pub const STRING: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_STRING);
    pub const REGISTER: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_REGISTER);
    pub const CATEGORY: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_CATEGORY);
    pub const ENUMERATION: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_ENUMERATION);
    pub const ENUM_ENTRY: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_ENUM_ENTRY);
    pub const PORT: Self = Self(sys::PylonNodeType::PYLON_NODE_TYPE_PORT);
}

/// `GenApi::EAccessMode`; pylon may report values outside the constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessMode(sys::PylonAccessMode);

impl AccessMode {
    /// Not implemented.
    pub const NI: Self = Self(sys::PylonAccessMode::PYLON_ACCESS_MODE_NI);
    /// Not available.
    pub const NA: Self = Self(sys::PylonAccessMode::PYLON_ACCESS_MODE_NA);
    pub const WO: Self = Self(sys::PylonAccessMode::PYLON_ACCESS_MODE_WO);
    pub const RO: Self = Self(sys::PylonAccessMode::PYLON_ACCESS_MODE_RO);
    pub const RW: Self = Self(sys::PylonAccessMode::PYLON_ACCESS_MODE_RW);

    /// `GenApi::IsReadable`.
    pub fn is_readable(self) -> bool {
        self == Self::RO || self == Self::RW
    }

    /// `GenApi::IsWritable`.
    pub fn is_writable(self) -> bool {
        self == Self::WO || self == Self::RW
    }

    /// `GenApi::IsAvailable`.
    pub fn is_available(self) -> bool {
        self != Self::NI && self != Self::NA
    }
}
