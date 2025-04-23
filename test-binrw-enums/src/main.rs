use binrw::{BinRead, binrw, io::Cursor};

#[derive(PartialEq, Eq, Debug)]
#[binrw]
#[brw(little)]
struct Point(i16, i16);

#[derive(BinRead, PartialEq, Eq, Debug)]
#[br(big)]
enum Shape {
    #[br(magic(0u8))] Rect {
        left: i16, top: i16, right: i16, bottom: i16
    },
    #[br(magic(1u8))] Oval { origin: Point, rx: u8, ry: u8 }
}

fn main() {
    let oval = Shape::read(&mut Cursor::new(b"\x01\x80\x02\xe0\x01\x2a\x15")).unwrap();
    assert_eq!(oval, Shape::Oval { origin: Point(640, 480), rx: 42, ry: 21 });
}