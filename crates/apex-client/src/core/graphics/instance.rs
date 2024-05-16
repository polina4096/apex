use bytemuck::{Pod, Zeroable};

pub trait Instance {
    type Baked: Pod + Zeroable;

    fn bake(&self) -> Self::Baked;
}
