pub type NodeId = usize;
pub type EdgeId = usize;
pub type Weight = usize;
pub type Rank = usize;
pub type GridId = usize;

pub const INVALID_NODE: NodeId = std::usize::MAX;
pub const WEIGHT_MAX: Weight = std::usize::MAX;
pub const INVALID_RANK: Rank = std::usize::MAX;

pub const DIST_MULTIPLICATOR: usize = 262144; // 2^18

// ratio: north south 876km / west east 640 km ~ 100:136
pub const LAT_GRID_AMOUNT: usize = 136;
pub const LNG_GRID_AMOUNT: usize = 100;
