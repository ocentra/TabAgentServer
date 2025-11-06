// Core trait and type modules
pub mod graph_traits;
pub mod algo;
pub mod prelude;

// Algorithm modules - all rewritten for zero-copy MDBX+rkyv
pub mod articulation_points;
pub mod astar;
pub mod bellman_ford;
pub mod bridges;
pub mod coloring;
pub mod community_detection;
pub mod dijkstra;
pub mod dominators;
pub mod feedback_arc_set;
pub mod flow_algorithms;
pub mod floyd_warshall;
pub mod isomorphism;
pub mod johnson;
pub mod k_shortest_path;
pub mod matching;
pub mod maximal_cliques;
pub mod maximum_flow;
pub mod min_spanning_tree;
pub mod page_rank;
pub mod scc;
pub mod simple_paths;
pub mod spfa;
pub mod steiner_tree;
pub mod traversal;
pub mod tred;

use std::vec::Vec;

use crate::visit::*;
use crate::unionfind::UnionFind;

// Re-export core algorithm traits
pub use algo::{Measure, BoundedMeasure, PositiveMeasure, UnitMeasure, FloatMeasure, Infinity, NegativeCycle};

// Re-export zero-copy implementations
pub use astar::astar_zero_copy;
pub use bellman_ford::bellman_ford_zero_copy;
pub use bridges::bridges_zero_copy;
pub use coloring::dsatur_coloring_zero_copy;
pub use community_detection::louvain_zero_copy;
pub use dijkstra::dijkstra_zero_copy;
pub use dominators::dominators_zero_copy;
pub use feedback_arc_set::feedback_arc_set_zero_copy;
pub use flow_algorithms::{max_flow_zero_copy, min_cost_max_flow_zero_copy};
pub use floyd_warshall::floyd_warshall_zero_copy;
pub use isomorphism::is_isomorphic_zero_copy;
pub use johnson::johnson_zero_copy;
pub use k_shortest_path::k_shortest_paths_zero_copy;
pub use matching::max_weight_matching_zero_copy;
pub use maximal_cliques::bron_kerbosch_zero_copy;
pub use maximum_flow::{dinics_zero_copy, ford_fulkerson_max_flow_zero_copy};
pub use min_spanning_tree::{kruskal_mst_zero_copy, prim_mst_zero_copy};
pub use page_rank::page_rank_zero_copy;
pub use scc::{kosaraju_scc, tarjan_scc};
pub use simple_paths::all_simple_paths_zero_copy;
pub use spfa::spfa_zero_copy;
pub use steiner_tree::steiner_tree_zero_copy;
pub use traversal::{bfs_zero_copy, dfs_zero_copy};
pub use tred::{transitive_closure_zero_copy, transitive_reduction_zero_copy};
pub use articulation_points::articulation_points_zero_copy;

type DfsSpaceType<G> = DfsSpace<<G as GraphBase>::NodeId, <G as Visitable>::Map>;

/// Workspace for a graph traversal.
#[derive(Clone, Debug)]
pub struct DfsSpace<N, VM> {
    dfs: Dfs<N, VM>,
}

impl<N, VM> DfsSpace<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    pub fn new<G>(g: G) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        DfsSpace { dfs: Dfs::empty(g) }
    }
}

impl<N, VM> Default for DfsSpace<N, VM>
where
    VM: VisitMap<N> + Default,
{
    fn default() -> Self {
        DfsSpace {
            dfs: Dfs {
                stack: <_>::default(),
                discovered: <_>::default(),
            },
        }
    }
}

fn with_dfs<G, F, R>(g: G, space: Option<&mut DfsSpaceType<G>>, f: F) -> R
where
    G: GraphRef + Visitable,
    F: FnOnce(&mut Dfs<G::NodeId, G::Map>) -> R,
{
    let mut local_visitor;
    let dfs = if let Some(v) = space {
        &mut v.dfs
    } else {
        local_visitor = Dfs::empty(g);
        &mut local_visitor
    };
    f(dfs)
}

/// Return the number of connected components of the graph.
pub fn connected_components<G>(g: G) -> usize
where
    G: NodeCompactIndexable + IntoEdgeReferences,
{
    let mut node_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());
        node_sets.union(g.to_index(a), g.to_index(b));
    }

    let mut labels = node_sets.into_labeling();
    labels.sort_unstable();
    labels.dedup();
    labels.len()
}

/// Return `true` if the input graph contains a cycle (undirected).
pub fn is_cyclic_undirected<G>(g: G) -> bool
where
    G: NodeIndexable + IntoEdgeReferences,
{
    let mut edge_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());
        if !edge_sets.union(g.to_index(a), g.to_index(b)) {
            return true;
        }
    }
    false
}

/// Perform a topological sort of a directed graph.
pub fn toposort<G>(
    g: G,
    space: Option<&mut DfsSpace<G::NodeId, G::Map>>,
) -> Result<Vec<G::NodeId>, Cycle<G::NodeId>>
where
    G: IntoNeighborsDirected + IntoNodeIdentifiers + Visitable,
{
    with_dfs(g, space, |dfs| {
        dfs.reset(g);
        let mut finished = g.visit_map();

        let mut finish_stack = Vec::new();
        for i in g.node_identifiers() {
            if dfs.discovered.is_visited(&i) {
                continue;
            }
            dfs.stack.push(i);
            while let Some(&nx) = dfs.stack.last() {
                if dfs.discovered.visit(nx) {
                    for succ in g.neighbors(nx) {
                        if succ == nx {
                            return Err(Cycle(nx));
                        }
                        if !dfs.discovered.is_visited(&succ) {
                            dfs.stack.push(succ);
                        }
                    }
                } else {
                    dfs.stack.pop();
                    if finished.visit(nx) {
                        finish_stack.push(nx);
                    }
                }
            }
        }
        finish_stack.reverse();

        dfs.reset(g);
        for &i in &finish_stack {
            dfs.move_to(i);
            let mut cycle = false;
            while let Some(j) = dfs.next(Reversed(g)) {
                if cycle {
                    return Err(Cycle(j));
                }
                cycle = true;
            }
        }

        Ok(finish_stack)
    })
}

/// Return `true` if the input directed graph contains a cycle.
pub fn is_cyclic_directed<G>(g: G) -> bool
where
    G: IntoNodeIdentifiers + IntoNeighbors + Visitable,
{
    use crate::visit::{depth_first_search, DfsEvent};

    depth_first_search(g, g.node_identifiers(), |event| match event {
        DfsEvent::BackEdge(_, _) => Err(()),
        _ => Ok(()),
    })
    .is_err()
}

/// Check if there exists a path starting at `from` and reaching `to`.
pub fn has_path_connecting<G>(
    g: G,
    from: G::NodeId,
    to: G::NodeId,
    space: Option<&mut DfsSpace<G::NodeId, G::Map>>,
) -> bool
where
    G: IntoNeighbors + Visitable,
{
    with_dfs(g, space, |dfs| {
        dfs.reset(g);
        dfs.move_to(from);
        dfs.iter(g).any(|x| x == to)
    })
}

/// An algorithm error: a cycle was found in the graph.
#[derive(Clone, Debug, PartialEq)]
pub struct Cycle<N>(pub(crate) N);

impl<N> Cycle<N> {
    pub fn node_id(&self) -> N
    where
        N: Copy,
    {
        self.0
    }
}

/// Return `true` if the graph is bipartite.
pub fn is_bipartite_undirected<G, N, VM>(g: G, start: N) -> bool
where
    G: GraphRef + Visitable<NodeId = N, Map = VM> + IntoNeighbors<NodeId = N>,
    N: Copy + PartialEq + core::fmt::Debug,
    VM: VisitMap<N>,
{
    let mut red = g.visit_map();
    red.visit(start);
    let mut blue = g.visit_map();

    let mut stack = std::collections::VecDeque::new();
    stack.push_front(start);

    while let Some(node) = stack.pop_front() {
        let is_red = red.is_visited(&node);
        let is_blue = blue.is_visited(&node);

        assert!(is_red ^ is_blue);

        for neighbour in g.neighbors(node) {
            let is_neigbour_red = red.is_visited(&neighbour);
            let is_neigbour_blue = blue.is_visited(&neighbour);

            if (is_red && is_neigbour_red) || (is_blue && is_neigbour_blue) {
                return false;
            }

            if !is_neigbour_red && !is_neigbour_blue {
                match (is_red, is_blue) {
                    (true, false) => {
                        blue.visit(neighbour);
                    }
                    (false, true) => {
                        red.visit(neighbour);
                    }
                    (_, _) => {
                        panic!("Invariant doesn't hold");
                    }
                }

                stack.push_back(neighbour);
            }
        }
    }

    true
}
