#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SemaphoreUsage {
    ImageAvailable(usize),
    RenderFinished(usize),
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FenceUsage {
    CommandBufferExec(usize),
    ImageAvailable(usize),
}
