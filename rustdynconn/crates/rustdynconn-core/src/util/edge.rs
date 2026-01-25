pub type EdgeKey = (u32, u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeRec {
    pub u: u32,
    pub v: u32,
}

pub fn canonical_edge(u: u32, v: u32) -> EdgeKey {
    if u <= v {
        (u, v)
    } else {
        (v, u)
    }
}
