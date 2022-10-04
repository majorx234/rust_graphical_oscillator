pub struct CtrlMsg {
    pub size: usize,
    pub intensity_am: f32,
    pub freq_am: f32,
    pub phase_am: f32,
    pub intensity_fm: f32,
    pub freq_fm: f32,
    pub phase_fm: f32,
    pub num_samples: usize,
}
