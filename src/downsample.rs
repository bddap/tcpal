/// Downsample src to dst using linear interpolation.
///
/// Panics if dst is larger than dst.
pub fn downsample(src: &[f32], dst: &mut [f32]) {
    let dstlen = dst.len();
    for (i, d) in dst.iter_mut().enumerate() {
        *d = interpsample(src, loc(dstlen, i));
    }
}

// panics in debug if arr_len == 1 or arr_len < index
fn loc(arr_len: usize, index: usize) -> f32 {
    debug_assert!(arr_len > index);
    debug_assert_ne!(arr_len, 1);
    (index as f32) / ((arr_len as f32) - 1.0)
}

/// if src is is splayed over a number line, loc is the location on that line
/// the last element of src is at loc = 1.0. The first is at 0
///
/// src.len() == 0 will result in a panic in debug, and unknown behavour in release
fn interpsample(src: &[f32], loc: f32) -> f32 {
    let srcmax = (src.len() - 1) as f32; // maxim legal index for src
    let underf: f32 = (loc * srcmax).floor();
    let overf: f32 = (loc * srcmax).ceil();
    let under = underf as usize;
    let over = overf as usize;

    debug_assert!(under < src.len());
    debug_assert!(over < src.len());

    // if under == over It means that loc happens to be a whole number
    // floor == ceil for whole numbers. The following guards against that case
    if over == under {
        // probably taking a bit of a performance hit here by branching
        return src[over];
    }

    let slope = (src[over] - src[under]) / (overf - underf);
    let xoffset = underf;
    let yoffset = src[under];

    ((loc * srcmax) - xoffset) * slope + yoffset
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.00001
    }

    #[test]
    fn tintersample() {
        let src = [3.0, 6.0, 2.0];
        dbg!(interpsample(&src, 0.0));
        assert!(near(interpsample(&src, 0.0), 3.0));
    }

    #[test]
    fn tloc() {
        for i in 2..1000 {
            assert!(loc(i, i - 1) <= 1.0);
            assert!(loc(i, 0) >= 0.0);
        }
    }

    #[test]
    fn upsample() {
        let src = [3.0, 6.0, 2.0];
        let mut dest = [0.0; 5];
        let expected = [3.0, 4.5, 6.0, 4.0, 2.0];
        downsample(&src, &mut dest);
        dbg!(dest);
        for (e, d) in expected.iter().zip(&dest) {
            assert!(near(*e, *d));
        }
    }

    #[test]
    fn tdownsample() {
        let src = [3.0, 6.0, -4.0, 2.0];
        let mut dest = [0.0; 3];
        let expected = [3.0, 1.0, 2.0];
        downsample(&src, &mut dest);
        dbg!(dest);
        for (e, d) in expected.iter().zip(&dest) {
            assert!(near(*e, *d));
        }
    }
}
