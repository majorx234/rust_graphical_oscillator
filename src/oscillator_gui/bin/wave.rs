use crate::ctrl_msg::CtrlMsg;

pub trait Wave: Send {
    fn new(frame_size: u32, sample_rate: f32) -> Self
    where
        Self: Sized;
    fn process_samples(&mut self, output_l: &mut [f32], output_r: &mut [f32]) {}
    fn ctrl(&mut self, msg: &CtrlMsg) {}
}
