pub mod segment;
pub mod segmentation;

#[cfg(test)]
mod tests {
    use crate::segment::*;

    #[test]
    fn counting_test1() {
        let img = [true, false, false,
                              false, true, false,
                              false, false, true];
        let segment = Segment::new((0, 0), (2, 2));
        let count = segment.count_in_pixels(&img, 3);
        assert_eq!(count, 3);
    }

    #[test]
    fn counting_test2() {
        let img = [true, true, false, false,
                              false, false, true, true,
                              false, false, false, false,
                              false, false, false, false];
        let segment = Segment::new((0, 0), (3, 1));
        let count = segment.count_in_pixels(&img, 4);
        assert_eq!(count, 3);
        let segment1 = Segment::new((0, 2), (3, 3));
        let count1 = segment1.count_in_pixels(&img, 4);
        assert_eq!(count1, 0);
    }

    #[test]
    fn counting_test3() {
        let img = [true, false, false, false, false,
                               true, false, true, true, false,
                               false, true, false, false, false,
                               false, true, false, false, false,
                               false, false, true, false, false];
        let segment = Segment::new((0, 0), (2, 4));
        let count = segment.count_in_pixels(&img, 5);
        assert_eq!(count, 5);
        let segment1 = Segment::new((0, 2), (4, 3));
        let count1 = segment1.count_in_pixels(&img, 5);
        assert_eq!(count1, 1);
    }

    #[test]
    fn counting_test4() {
        let img = [true, false, false, true, true, true,
                               true, true, true, true, false, false,
                               false, true, true, false, false, false,
                               false, true, true, false, false, false,
                               false, true, true, false, false, false,
                               false, true, true, false, false, false];
        let segment = Segment::new((5, 0), (0, 1));
        let count = segment.count_in_pixels(&img, 6);
        //println!("{}", count);
        assert_eq!(count, 4);
        let segment1 = Segment::new((3, 1), (1, 5));
        let count1 = segment1.count_in_pixels(&img, 6);
        //println!("{}", count1);
        assert_eq!(count1, 5);
    }

}