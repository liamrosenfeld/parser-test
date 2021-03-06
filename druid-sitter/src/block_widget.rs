use crate::Model;
use druid::widget::Painter;
use druid::{Color, PaintCtx, Point, Rect, RenderContext, Size};
use tree_sitter::Node;

pub fn make_blocks() -> Painter<Model> {
    Painter::new(|ctx, data: &Model, _env| {
        // pre-order traversal because we want to draw the parent under their children
        let mut cursor = data.tree.as_ref().root_node().walk();
        'outer: loop {
            // first time encountering the node, so draw it
            draw_node(cursor.node(), &data.source, ctx);

            // keep traveling down the tree as far as we can
            if cursor.goto_first_child() {
                continue;
            }

            // if we can't travel any further down, try the next sibling
            if cursor.goto_next_sibling() {
                continue;
            }

            // travel back up
            // loop until we reach the root or can go to the next sibling of a node again
            'inner: loop {
                // break outer if we reached the root
                if !cursor.goto_parent() {
                    break 'outer;
                }

                // if there is a sibling at this level, visit the sibling's subtree
                if cursor.goto_next_sibling() {
                    break 'inner;
                }
            }
        }
    })
}

/*
Got these values by running:
    let font = FontDescriptor::new(FontFamily::new_unchecked("Roboto Mono")).with_size(15.0);
    let mut layout = TextLayout::<String>::from_text("A".to_string());
    layout.set_font(font);
    layout.rebuild_if_needed(ctx.text(), env);
    let size = layout.size();
    println!("{:}", size);
*/
const FONT_WIDTH: f64 = 9.00146484375;
const FONT_HEIGHT: f64 = 20.0;

fn draw_node(node: Node, source: &str, ctx: &mut PaintCtx) {
    // don't draw boxes for nodes that are just string literals
    // without this every space would get it's own color
    if !node.is_named() {
        return;
    }

    // get color/see if this node should be drawn
    // don't draw boxes for nodes that aren't high level
    let color = match color(&node) {
        Some(color) => color,
        None => return,
    };

    let start = node.start_position();
    let end = node.end_position();

    let start_pt = Point::new(
        (start.column as f64) * FONT_WIDTH,
        (start.row as f64) * FONT_HEIGHT,
    );

    let size = {
        if start.row == end.row {
            // if block is all on one row, then
            let width = ((end.column - start.column) as f64) * FONT_WIDTH;
            Size::new(width, FONT_HEIGHT)
        } else {
            // if block is across rows,
            // then the end column won't necessarily be the furthest point to the left
            // this will also fix an out of bounds if start > end col
            let height = ((end.row - start.row + 1) as f64) * FONT_HEIGHT;
            // find the longest line of the block
            let columns = source[node.byte_range()]
                .lines()
                .map(|l| l.len())
                .max()
                .unwrap_or(0);
            Size::new(columns as f64 * FONT_WIDTH, height)
        }
    };

    // draw the block in
    let block = Rect::from_origin_size(start_pt, size);
    ctx.fill(block, &color);
}

fn color(node: &Node) -> Option<Color> {
    match node.kind() {
        "module" => Some(Color::rgb(0.0, 0.4, 0.4)),
        "class_definition" => Some(Color::rgb(0.9, 0.43, 0.212)),
        "function_definition" => Some(Color::rgb(0.0, 0.47, 0.47)),
        "import_statement" => Some(Color::rgb(0.77, 0.176, 0.188)),
        "expression_statement" => Some(Color::rgb(0.5, 0.2, 0.5)),
        "while_statement" => Some(Color::rgb(0.305, 0.0, 0.305)),
        "if_statement" => Some(Color::rgb(0.502, 0.086, 0.22)),
        _ => None,
    }
}
