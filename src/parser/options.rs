mod flags {
    pub const TRACK_IDS: u8 = 1 << 0;
    pub const TRACK_CLASSES: u8 = 1 << 1;
    pub const HIGHEST: u8 = TRACK_CLASSES;
}

/// Options for the HTML Parser
///
/// This allows users of this library to configure the parser.
/// The default options (`ParserOptions::default()`) are optimized for raw parsing.
/// If you need to do HTML tag lookups by ID or class names, you can enable tracking.
/// This will cache HTML nodes as they appear in the source code on the fly.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParserOptions {
    flags: u8,
    max_depth: usize,
}

// some reasonable default max depth
const MAX_DEFAULT_DEPTH: usize = 256;

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            flags: 0,
            max_depth: MAX_DEFAULT_DEPTH,
        }
    }
}

impl ParserOptions {
    /// Creates a new [ParserOptions] with no flags set
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [ParserOptions] from a bitset
    pub fn from_raw_checked(flags: u8) -> Option<Self> {
        if flags > flags::HIGHEST * 2 - 1 {
            None
        } else {
            Some(Self {
                flags,
                ..Default::default()
            })
        }
    }

    /// Returns the raw flags of this bitset
    pub fn to_raw(&self) -> u8 {
        self.flags
    }

    fn set_flag(&mut self, flag: u8) {
        self.flags |= flag;
    }

    #[inline]
    fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    /// Enables tracking of HTML Tag IDs and stores them in a lookup table.
    ///
    /// This makes `get_element_by_id()` lookups ~O(1)
    pub fn track_ids(mut self) -> Self {
        self.set_flag(flags::TRACK_IDS);
        self
    }

    /// Enables tracking of HTML Tag classes and stores them in a lookup table.
    ///
    /// This makes `get_elements_by_class_name()` lookups ~O(1)
    pub fn track_classes(mut self) -> Self {
        self.set_flag(flags::TRACK_CLASSES);
        self
    }

    /// Returns whether the parser is tracking HTML Tag IDs.
    #[inline]
    pub fn is_tracking_ids(&self) -> bool {
        self.has_flag(flags::TRACK_IDS)
    }

    /// Returns whether the parser is tracking HTML Tag classes.
    #[inline]
    pub fn is_tracking_classes(&self) -> bool {
        self.has_flag(flags::TRACK_CLASSES)
    }

    /// Returns the maximum depth of the HTML parser
    #[inline]
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Sets the maximum recursion depth this HTML parser is allowed to take
    ///
    /// By default, this is set to a reasonably small value to not blow up the stack.
    #[inline]
    pub fn set_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Returns whether the parser is tracking HTML Tag IDs or classes (previously enabled by a call to `track_ids()` or `track_classes()`).
    #[inline]
    pub fn is_tracking(&self) -> bool {
        // for now we can just check if any bit is set, may or may not lead to better codegen than two cmps
        // this must be changed in some way if we ever add more flags
        // self.is_tracking_ids() || self.is_tracking_classes()
        self.flags > 0
    }
}
