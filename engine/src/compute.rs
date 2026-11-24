//! The compute side of the engine (particle_system § 2 onwards). For now: how big a
//! dispatch has to be. § 3 adds compute pipelines.

/// How a 1D job of `count` items splits into workgroups of `local_size`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchSize {
    /// Workgroups to launch: `count ÷ local_size`, rounded **up** (whole groups only).
    pub workgroups: u32,
    /// Invocations that will run: `workgroups × local_size` (≥ `count`).
    pub invocations: u64,
    /// Invocations with no item to work on (`invocations − count`): the shader's
    /// bounds guard sends these home.
    pub idle: u32,
}

impl DispatchSize {
    /// Sizes a 1D dispatch. `local_size` must not be 0.
    pub fn for_items(count: u32, local_size: u32) -> Self {
        assert!(local_size > 0, "a workgroup needs at least one invocation");
        // ceil(count / local_size) with integers only. `div_ceil` is the section's
        // (count + local_size − 1) / local_size, done without overflowing; u64 so the
        // total (workgroups × local_size) fits even for a count near u32::MAX.
        let workgroups = u64::from(count).div_ceil(u64::from(local_size));
        let invocations = workgroups * u64::from(local_size);
        DispatchSize {
            workgroups: workgroups as u32,
            invocations,
            idle: (invocations - u64::from(count)) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_up_to_whole_workgroups() {
        let d = DispatchSize::for_items(1_000_000, 256);
        assert_eq!(
            (d.workgroups, d.invocations, d.idle),
            (3907, 1_000_192, 192)
        );
        let d = DispatchSize::for_items(10, 4);
        assert_eq!((d.workgroups, d.invocations, d.idle), (3, 12, 2));
    }

    #[test]
    fn exact_multiples_have_no_idle_invocations() {
        let d = DispatchSize::for_items(2_000_000, 64);
        assert_eq!((d.workgroups, d.idle), (31_250, 0));
        assert_eq!(DispatchSize::for_items(1_048_576, 256).idle, 0);
    }

    #[test]
    fn zero_items_need_no_workgroups() {
        assert_eq!(DispatchSize::for_items(0, 256).workgroups, 0);
    }

    #[test]
    fn huge_counts_do_not_overflow() {
        let d = DispatchSize::for_items(u32::MAX, 256);
        assert_eq!(d.workgroups, 16_777_216);
        assert_eq!(d.idle, 1);
    }
}
