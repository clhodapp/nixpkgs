//! Leaving out the imports that other imports make unnecessary.
//!
//! The modules and their imports are a directed graph. Some imports are
//! fixed: a person wrote them. The others are the ones the tool may
//! write. The task is to keep as few of those as possible while every
//! module still reaches every module it reached before. This is the
//! transitive reduction of the graph (Aho, Garey and Ullman, 1972),
//! with the fixed imports for free.
//!
//! The modules are first put into groups that reach each other, the
//! strongly connected components of the graph. Between the groups the
//! graph has no cycle, and there the smallest set of imports is unique
//! and found exactly. Inside a group the task is the minimum strongly
//! connected spanning subgraph, which is NP-hard in general. The groups
//! of a module tree are small, so the tool searches for the smallest
//! set, and it reports a lower bound with the result.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

/// The imports of each module, by the number of the module.
pub type Edges = Vec<BTreeSet<usize>>;

/// A group of modules that reach each other.
pub struct Group {
    /// The number of modules in the group.
    pub modules: usize,
    /// The imports inside the group that the tool keeps.
    pub kept: usize,
    /// Whether the search for the smallest set finished in time.
    pub searched: bool,
    /// No set of imports for the group is smaller than this.
    pub lower_bound: usize,
}

/// Whether the module `from` reaches the module `to` along the imports.
pub fn reaches(edges: &[BTreeSet<usize>], from: usize, to: usize) -> bool {
    let mut seen = vec![false; edges.len()];
    let mut stack = vec![from];
    seen[from] = true;
    while let Some(node) = stack.pop() {
        for &next in &edges[node] {
            if next == to {
                return true;
            }
            if !seen[next] {
                seen[next] = true;
                stack.push(next);
            }
        }
    }
    false
}

/// The group of each module, with the algorithm of Tarjan for strongly
/// connected components. The walk keeps its own stack, because a chain
/// of imports can be as long as the tree has modules.
fn components(edges: &[BTreeSet<usize>]) -> Vec<usize> {
    const UNSEEN: usize = usize::MAX;
    let count = edges.len();
    let mut index = vec![UNSEEN; count];
    let mut low = vec![0; count];
    let mut component = vec![UNSEEN; count];
    let mut on_stack = vec![false; count];
    let mut stack = Vec::new();
    let mut next_index = 0;
    let mut next_component = 0;
    for root in 0..count {
        if index[root] != UNSEEN {
            continue;
        }
        let mut work: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut enter = |node: usize,
                         work: &mut Vec<(usize, Vec<usize>)>,
                         index: &mut Vec<usize>,
                         low: &mut Vec<usize>,
                         stack: &mut Vec<usize>,
                         on_stack: &mut Vec<bool>| {
            index[node] = next_index;
            low[node] = next_index;
            next_index += 1;
            stack.push(node);
            on_stack[node] = true;
            work.push((node, edges[node].iter().rev().copied().collect()));
        };
        enter(
            root,
            &mut work,
            &mut index,
            &mut low,
            &mut stack,
            &mut on_stack,
        );
        while let Some((node, successors)) = work.last_mut() {
            let node = *node;
            if let Some(successor) = successors.pop() {
                if index[successor] == UNSEEN {
                    enter(
                        successor,
                        &mut work,
                        &mut index,
                        &mut low,
                        &mut stack,
                        &mut on_stack,
                    );
                } else if on_stack[successor] {
                    low[node] = low[node].min(index[successor]);
                }
                continue;
            }
            work.pop();
            if let Some((parent, _)) = work.last() {
                low[*parent] = low[*parent].min(low[node]);
            }
            if low[node] == index[node] {
                while let Some(member) = stack.pop() {
                    on_stack[member] = false;
                    component[member] = next_component;
                    if member == node {
                        break;
                    }
                }
                next_component += 1;
            }
        }
    }
    component
}

/// A number that no set of imports for a group is smaller than. `inside`
/// are the imports between the modules of the group. Only the imports
/// in `removable` count, because the others are there anyway.
///
/// Every module of the group has an import that enters it and one that
/// leaves it. An import that is the only one to enter a module, or the
/// only one to leave a module, is in every set. Counted by the module
/// that an import leaves, a module therefore has all such imports that
/// leave it, and one import at least. The same holds counted by the
/// module that an import enters.
fn lower_bound(
    group: &[usize],
    inside: &[(usize, usize)],
    removable: &BTreeSet<(usize, usize)>,
) -> usize {
    let mut leaving: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    let mut entering: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    for &import in inside {
        leaving.entry(import.0).or_default().push(import);
        entering.entry(import.1).or_default().push(import);
    }
    let in_every_set: BTreeSet<(usize, usize)> = leaving
        .values()
        .chain(entering.values())
        .filter(|imports| imports.len() == 1)
        .map(|imports| imports[0])
        .filter(|import| removable.contains(import))
        .collect();
    let counted_by = |imports_of: &BTreeMap<usize, Vec<(usize, usize)>>| -> usize {
        group
            .iter()
            .map(|module| {
                let imports = imports_of.get(module).map_or(&[][..], Vec::as_slice);
                let needed = imports
                    .iter()
                    .filter(|import| in_every_set.contains(import))
                    .count();
                let has_fixed = imports.iter().any(|import| !removable.contains(import));
                if has_fixed { needed } else { needed.max(1) }
            })
            .sum()
    };
    counted_by(&leaving).max(counted_by(&entering))
}

/// The imports that the tool keeps inside one group.
struct Chosen {
    keep: BTreeSet<(usize, usize)>,
    /// Whether `keep` is a smallest set. It is not when the time for
    /// the search was up.
    smallest: bool,
    /// No set is smaller than this, as far as the search found out.
    bound: usize,
}

/// The smallest set of removable imports that lets every module of a
/// group reach every other one, together with the fixed imports.
///
/// This is an integer program with one variable for each removable
/// import, which is 1 when the import stays. For a set of modules that
/// is not the whole group, one import has to leave the set and one has
/// to enter it. The program starts with these two conditions for each
/// single module. The answer is then looked at: where its modules fall
/// into several sets that do not reach each other, the two conditions
/// for each such set are added, and the program is solved again. The
/// first answer in which all modules reach each other is a smallest
/// set, because it is the best answer under conditions that every
/// valid set meets.
///
/// When the time is up at `deadline`, the work so far is not thrown
/// away. The imports that the search has decided on stay decided. The
/// last answer of the program is a set that is as small as any valid
/// one and that leaves some modules cut off. The result is built from
/// it: the imports outside that answer are taken out one at a time
/// first, and one stays only when its file would go out of reach. The
/// imports of the answer are tried after them. The result has no
/// import that could be left out, and it is close to the answer.
fn smallest(
    group: &[usize],
    inside: &[(usize, usize)],
    removable: &BTreeSet<(usize, usize)>,
    deadline: Instant,
) -> Chosen {
    use microlp::{ComparisonOp, LinearExpr, OptimizationDirection, Problem};

    let local: BTreeMap<usize, usize> = group
        .iter()
        .enumerate()
        .map(|(index, &module)| (module, index))
        .collect();
    let mut candidates: Vec<(usize, usize)> = inside
        .iter()
        .copied()
        .filter(|import| removable.contains(import))
        .collect();
    let mut fixed: Vec<(usize, usize)> = inside
        .iter()
        .copied()
        .filter(|import| !removable.contains(import))
        .collect();

    // Much is decided before the search. An import that is the only one
    // to leave a module, or the only one to enter it, is in every set,
    // and from here on it counts as fixed. An import of a file that the
    // module reaches along fixed imports is in no smallest set. Each of
    // the two rules can make the other one apply again.
    let mut in_every_set: BTreeSet<(usize, usize)> = BTreeSet::new();
    loop {
        let mut along_fixed: Edges = vec![BTreeSet::new(); group.len()];
        for import in &fixed {
            along_fixed[local[&import.0]].insert(local[&import.1]);
        }
        let before = candidates.len();
        candidates.retain(|import| !reaches(&along_fixed, local[&import.0], local[&import.1]));
        let mut found: BTreeSet<(usize, usize)> = BTreeSet::new();
        for &module in group {
            for leaving in [true, false] {
                let end = |import: &(usize, usize)| if leaving { import.0 } else { import.1 };
                if fixed.iter().any(|import| end(import) == module) {
                    continue;
                }
                let mut only = candidates.iter().filter(|import| end(import) == module);
                if let (Some(import), None) = (only.next(), only.next()) {
                    found.insert(*import);
                }
            }
        }
        if found.is_empty() && candidates.len() == before {
            break;
        }
        candidates.retain(|import| !found.contains(import));
        fixed.extend(found.iter().copied());
        in_every_set.extend(found);
    }
    // The sets of modules that one import has to leave and to enter.
    let mut sets: Vec<Vec<bool>> = (0..group.len())
        .map(|member| (0..group.len()).map(|index| index == member).collect())
        .collect();

    // The last answer of the program that left modules cut off.
    let mut last: Option<BTreeSet<(usize, usize)>> = None;
    'search: loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        problem.set_time_limit(left);
        let variables: Vec<_> = (0..candidates.len())
            .map(|_| problem.add_binary_var(1.0))
            .collect();
        for set in &sets {
            // Leaving the set, then entering it.
            for leaving in [true, false] {
                let crosses = |import: &(usize, usize)| {
                    let (from, to) = (set[local[&import.0]], set[local[&import.1]]);
                    if leaving { from && !to } else { !from && to }
                };
                if fixed.iter().any(crosses) {
                    continue;
                }
                let mut expression = LinearExpr::empty();
                let mut any = false;
                for (index, import) in candidates.iter().enumerate() {
                    if crosses(import) {
                        expression.add(variables[index], 1.0);
                        any = true;
                    }
                }
                if !any {
                    break 'search;
                }
                problem.add_constraint(expression, ComparisonOp::Ge, 1.0);
            }
        }
        let Ok(outcome) = problem.solve() else {
            break;
        };
        if !outcome.is_optimal() {
            break;
        }
        let Ok(solution) = outcome.into_solution() else {
            break;
        };
        let mut keep: BTreeSet<(usize, usize)> = candidates
            .iter()
            .enumerate()
            .filter(|(index, _)| solution.var_value_raw(variables[*index]) > 0.5)
            .map(|(_, import)| *import)
            .collect();

        let mut chosen: Edges = vec![BTreeSet::new(); group.len()];
        for import in fixed.iter().chain(keep.iter()) {
            chosen[local[&import.0]].insert(local[&import.1]);
        }
        let component = components(&chosen);
        let parts = component.iter().max().map_or(0, |last| last + 1);
        if parts == 1 {
            keep.extend(in_every_set);
            return Chosen {
                bound: keep.len(),
                keep,
                smallest: true,
            };
        }
        for part in 0..parts {
            sets.push(component.iter().map(|&of| of == part).collect());
        }
        last = Some(keep);
    }

    // The time is up. Build a set from what the search has.
    let preferred = last.unwrap_or_default();
    let bound = in_every_set.len() + preferred.len();
    let mut graph: Edges = vec![BTreeSet::new(); group.len()];
    for import in fixed.iter().chain(candidates.iter()) {
        graph[local[&import.0]].insert(local[&import.1]);
    }
    let outside_the_answer = candidates
        .iter()
        .filter(|import| !preferred.contains(import));
    let in_the_answer = candidates
        .iter()
        .filter(|import| preferred.contains(import));
    let mut keep = in_every_set;
    for import in outside_the_answer.chain(in_the_answer) {
        let (from, to) = (local[&import.0], local[&import.1]);
        graph[from].remove(&to);
        if !reaches(&graph, from, to) {
            graph[from].insert(to);
            keep.insert(*import);
        }
    }
    Chosen {
        keep,
        smallest: false,
        bound,
    }
}

/// Take out of `edges` every import in `removable` that is not needed
/// for the modules to reach what they reached before. Returns the
/// groups of more than one module.
///
/// The search for the smallest set inside the groups stops at
/// `deadline`. A group whose search has not finished by then gets a set
/// that has no import that could be left out, and that may not be the
/// smallest.
pub fn reduce(
    edges: &mut Edges,
    removable: &BTreeSet<(usize, usize)>,
    deadline: Instant,
) -> Vec<Group> {
    let component = components(edges);
    let groups = component.iter().max().map_or(0, |last| last + 1);

    // Between the groups. The graph of the groups has no cycle. An
    // import from one group to another is needed exactly when no way
    // leads from the first group to the second through a third group,
    // and then one import between the two groups is enough.
    let mut between: Edges = vec![BTreeSet::new(); groups];
    let mut by_pair: BTreeMap<(usize, usize), Vec<(usize, usize)>> = BTreeMap::new();
    for (from, targets) in edges.iter().enumerate() {
        for &to in targets {
            let pair = (component[from], component[to]);
            if pair.0 != pair.1 {
                between[pair.0].insert(pair.1);
                by_pair.entry(pair).or_default().push((from, to));
            }
        }
    }
    for (&(first, second), imports) in &by_pair {
        let through_a_third = between[first]
            .iter()
            .any(|&third| third != second && (reaches(&between, third, second)));
        let fixed = imports.iter().any(|import| !removable.contains(import));
        let mut keep_one = !through_a_third && !fixed;
        for import in imports {
            if !removable.contains(import) {
                continue;
            }
            if keep_one {
                keep_one = false;
            } else {
                edges[import.0].remove(&import.1);
            }
        }
    }

    // Inside each group.
    let mut members: Vec<Vec<usize>> = vec![Vec::new(); groups];
    for (module, &group) in component.iter().enumerate() {
        members[group].push(module);
    }
    let mut result = Vec::new();
    // The small groups go first, so that a large group that uses up the
    // time does not keep them from being searched.
    members.retain(|group| group.len() > 1);
    members.sort_by_key(|group| (group.len(), group[0]));
    for group in &members {
        let mut inside: Vec<(usize, usize)> = Vec::new();
        for &from in group {
            for &to in &edges[from] {
                if component[to] == component[from] {
                    inside.push((from, to));
                }
            }
        }
        let candidates: Vec<(usize, usize)> = inside
            .iter()
            .copied()
            .filter(|import| removable.contains(import))
            .collect();

        let chosen = smallest(group, &inside, removable, deadline);
        for import in &candidates {
            if !chosen.keep.contains(import) {
                edges[import.0].remove(&import.1);
            }
        }
        result.push(Group {
            modules: group.len(),
            kept: chosen.keep.len(),
            searched: chosen.smallest,
            lower_bound: chosen
                .bound
                .max(lower_bound(group, &inside, removable))
                .min(chosen.keep.len()),
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(count: usize, imports: &[(usize, usize)]) -> Edges {
        let mut edges = vec![BTreeSet::new(); count];
        for &(from, to) in imports {
            edges[from].insert(to);
        }
        edges
    }

    fn imports_of(edges: &Edges) -> Vec<(usize, usize)> {
        edges
            .iter()
            .enumerate()
            .flat_map(|(from, targets)| targets.iter().map(move |&to| (from, to)))
            .collect()
    }

    /// A deadline that the searches of these tests do not reach.
    fn soon() -> Instant {
        Instant::now() + std::time::Duration::from_secs(60)
    }

    #[test]
    fn falls_back_when_the_time_is_up() {
        // The ring with an import across it from every module, and no
        // time to search. The result still lets every module reach
        // every other one, and it is marked as not searched.
        let mut imports = Vec::new();
        for module in 0..6 {
            imports.push((module, (module + 1) % 6));
            imports.push((module, (module + 3) % 6));
        }
        let mut edges = graph(6, &imports);
        let removable = imports.iter().copied().collect();
        let groups = reduce(&mut edges, &removable, Instant::now());
        assert!(!groups[0].searched);
        assert!(groups[0].lower_bound <= groups[0].kept);
        for from in 0..6 {
            for to in 0..6 {
                assert!(from == to || reaches(&edges, from, to));
            }
        }
    }

    /// Reduce a graph in which the tool may take out every import.
    fn reduced(count: usize, imports: &[(usize, usize)]) -> (Vec<(usize, usize)>, Vec<Group>) {
        let mut edges = graph(count, imports);
        let removable = imports.iter().copied().collect();
        let groups = reduce(&mut edges, &removable, soon());
        // Every module reaches what it reached before.
        let before = graph(count, imports);
        for from in 0..count {
            for to in 0..count {
                if from != to {
                    assert_eq!(reaches(&before, from, to), reaches(&edges, from, to));
                }
            }
        }
        (imports_of(&edges), groups)
    }

    #[test]
    fn leaves_out_an_import_that_a_chain_makes_unnecessary() {
        let (kept, groups) = reduced(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (2, 3)]);
        assert_eq!(kept, [(0, 1), (1, 2), (2, 3)]);
        assert!(groups.is_empty());
    }

    #[test]
    fn keeps_a_file_in_reach_of_modules_that_import_each_other() {
        // 0 and 1 import each other and both import 2. One import of 2
        // is enough, and one has to stay.
        let (kept, groups) = reduced(3, &[(0, 1), (0, 2), (1, 0), (1, 2)]);
        assert_eq!(kept, [(0, 1), (0, 2), (1, 0)]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].modules, 2);
        assert_eq!(groups[0].kept, 2);
        assert_eq!(groups[0].lower_bound, 2);
    }

    #[test]
    fn finds_the_smallest_set_inside_a_group() {
        // Six modules in a ring, with an import across the ring from
        // every module. Six modules need six imports at least, and the
        // ring has six.
        let mut imports = Vec::new();
        for module in 0..6 {
            imports.push((module, (module + 1) % 6));
            imports.push((module, (module + 3) % 6));
        }
        let (kept, groups) = reduced(6, &imports);
        assert_eq!(kept.len(), 6);
        assert_eq!(groups[0].kept, 6);
        assert_eq!(groups[0].lower_bound, 6);
    }

    #[test]
    fn keeps_one_import_between_two_groups() {
        // Two pairs of modules that import each other, with three
        // imports from the first pair to the second.
        let (kept, _) = reduced(4, &[(0, 1), (1, 0), (2, 3), (3, 2), (0, 2), (0, 3), (1, 3)]);
        assert_eq!(kept, [(0, 1), (0, 2), (1, 0), (2, 3), (3, 2)]);
    }

    #[test]
    fn does_not_take_out_a_fixed_import() {
        let mut edges = graph(3, &[(0, 1), (0, 2), (1, 2)]);
        // A person wrote the import of 2 into 0.
        let removable = [(0, 1), (1, 2)].into_iter().collect();
        reduce(&mut edges, &removable, soon());
        assert_eq!(imports_of(&edges), [(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn writes_no_import_next_to_a_fixed_one_between_two_groups() {
        let mut edges = graph(4, &[(0, 1), (1, 0), (2, 3), (3, 2), (0, 2), (1, 3)]);
        // A person wrote the import of 3 into 1.
        let removable = [(0, 1), (1, 0), (2, 3), (3, 2), (0, 2)]
            .into_iter()
            .collect();
        reduce(&mut edges, &removable, soon());
        assert_eq!(imports_of(&edges), [(0, 1), (1, 0), (1, 3), (2, 3), (3, 2)]);
    }
}
