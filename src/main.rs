use std::{collections::HashMap, fmt::Display, fs::read_to_string, str::FromStr};

use clap::Parser;
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, space1},
    multi::{separated_list0, separated_list1},
    Finish, IResult,
};
use pathfinding::directed::dijkstra;

/// The color of the edge between two nodes. A more generalized solution may have more colors or
/// may require altering the algorithm. We'll show a solution though for just two colors.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Color {
    Red,
    Blue,
    None,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "red"),
            Color::Blue => write!(f, "blue"),
            Color::None => write!(f, "none"),
        }
    }
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        match s {
            "red" => Color::Red,
            "blue" => Color::Blue,
            _ => Color::None,
        }
    }
}

/// An edge in the graph.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Edge {
    color: Color,
    node: String,
}

impl Edge {
    fn new(color: Color, node: String) -> Self {
        Edge { color, node }
    }
}

/// The puzzle itself. This is a graph represented as a HashMap where the key is the node and the
/// value is a list of edges. Each edge is a tuple of the color of the edge and the node it leads
/// to.
struct Puzzle {
    nodes: HashMap<String, Vec<Edge>>,
}

impl FromStr for Puzzle {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use the nom parser combinator library to parse the puzzle.
        match Puzzle::parse_puzzle(s).finish() {
            Ok((_, puzzle)) => Ok(puzzle),
            Err(nom::error::Error { input, code }) => Err(nom::error::Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

impl Puzzle {
    /// Parse a puzzle from a string.
    fn parse_puzzle(input: &str) -> IResult<&str, Puzzle> {
        let (input, nodes) = separated_list0(tag("\n"), Puzzle::parse_node)(input)?;
        Ok((
            input,
            Puzzle {
                nodes: nodes.into_iter().collect(),
            },
        ))
    }

    /// Parse a node from a string (e.g. "a red:b blue:a"). Used in parsing.
    fn parse_node(input: &str) -> IResult<&str, (String, Vec<Edge>)> {
        let (input, node) = alpha1(input)?;
        let (input, _) = space1(input)?;
        let (input, edges) = separated_list1(space1, Puzzle::parse_edge)(input)?;
        Ok((input, (node.to_string(), edges)))
    }

    /// Parse an edge from a string (e.g. "red:b"). Used in parsing.
    fn parse_edge(input: &str) -> IResult<&str, Edge> {
        let (input, color) = alpha1(input)?;
        let (input, _) = tag(":")(input)?;
        let (input, node) = alpha1(input)?;
        Ok((input, Edge::new(Color::from(color), node.to_string())))
    }

    /// Solve the puzzle by finding the shortest path from the start node to the end node.
    fn solve(&self, start: String, end: String) -> Option<Vec<Edge>> {
        // We start at the start node with no color.
        let start = Edge::new(Color::None, start);

        // We've reached the end node when the current node is the end node.
        let success = |state: &Edge| state.node == end;

        // The successors of a state are the nodes that are connected to the current node. We filter
        // out nodes that are the same color as the current state. Dijkstra's algorithm uses a
        // weighted graph, but we don't need to weight the edges in this case so we always return 1.
        let successors = |edge: &Edge| {
            let edges = self.nodes.get(&edge.node).unwrap();
            edges
                .iter()
                .filter_map(|Edge { color, node }| {
                    if *color == Color::None || edge.color != *color {
                        Some(Edge {
                            color: *color,
                            node: node.clone(),
                        })
                    } else {
                        None
                    }
                })
                .map(|edge| (edge, 1))
                .collect::<Vec<(Edge, usize)>>()
        };

        // Run Dijkstra's algorithm to find the shortest path.
        dijkstra::dijkstra(&start, successors, success).map(|(solution, _)| solution)
    }
}

/// Solve the wall puzzle. The wall puzzle is a graph where each node is a letter and each edge is
/// a color. The goal is to find the shortest path from the start node to the end node where no two
/// adjacent edges used are the same color.
#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// The input file containing the puzzle.
    file: String,

    /// The start node.
    start: String,

    /// The end node.
    end: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the input file and parse the puzzle.
    let cli = Cli::parse();
    let input = read_to_string(cli.file)?;
    let puzzle: Puzzle = input.parse()?;

    // Solve the puzzle and print the solution.
    match puzzle.solve(cli.start, cli.end) {
        Some(solution) => print_solution(solution),
        None => println!("No solution found"),
    }
    Ok(())
}

/// Print the solution to the wall puzzle.
fn print_solution(solution: Vec<Edge>) {
    let mut prev = None;
    let mut cur = None;
    for state in solution {
        prev.clone_from(&cur);
        cur = Some(state);
        if let (Some(prev), Some(cur)) = (prev.clone(), cur.clone()) {
            println!("{} ==({})=> {}", prev.node, cur.color, cur.node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(Color::from("red"), Color::Red);
        assert_eq!(Color::from("blue"), Color::Blue);
        assert_eq!(Color::from("green"), Color::None);
    }

    #[test]
    fn test_parse_graph() {
        let input = "a red:b blue:a\nc red:a blue:b";
        let graph: Puzzle = input.parse().unwrap();

        let mut expected = HashMap::new();
        expected.insert(
            "a".to_string(),
            vec![
                Edge::new(Color::Red, "b".to_string()),
                Edge::new(Color::Blue, "a".to_string()),
            ],
        );
        expected.insert(
            "c".to_string(),
            vec![
                Edge::new(Color::Red, "a".to_string()),
                Edge::new(Color::Blue, "b".to_string()),
            ],
        );

        assert_eq!(graph.nodes, expected);
    }

    #[test]
    fn test_solve_simple() {
        let input = "a red:b \nb blue:a";
        let puzzle: Puzzle = input.parse().unwrap();
        let solution = puzzle.solve("a".to_string(), "b".to_string());
        assert_eq!(
            solution,
            Some(vec![
                Edge {
                    color: Color::None,
                    node: "a".to_string()
                },
                Edge {
                    color: Color::Red,
                    node: "b".to_string()
                },
            ])
        );
    }

    #[test]
    fn test_solve_complex() {
        let input = include_str!("../wall-puzzle.txt");
        let puzzle: Puzzle = input.parse().unwrap();
        let solution = puzzle.solve("s".to_string(), "t".to_string());
        assert_eq!(
            solution,
            Some(vec![
                Edge {
                    color: Color::None,
                    node: "s".to_string()
                },
                Edge {
                    color: Color::Red,
                    node: "a".to_string()
                },
                Edge {
                    color: Color::Blue,
                    node: "b".to_string()
                },
                Edge {
                    color: Color::Red,
                    node: "c".to_string()
                },
                Edge {
                    color: Color::Blue,
                    node: "c".to_string()
                },
                Edge {
                    color: Color::Red,
                    node: "b".to_string()
                },
                Edge {
                    color: Color::Blue,
                    node: "a".to_string()
                },
                Edge {
                    color: Color::Red,
                    node: "e".to_string()
                },
                Edge {
                    color: Color::Blue,
                    node: "t".to_string()
                }
            ])
        );
    }
}
