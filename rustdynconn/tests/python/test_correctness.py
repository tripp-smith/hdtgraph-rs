from collections import deque

from rustdynconn import DynamicGraph


def _has_path(edges: set[tuple[object, object]], start: object, target: object) -> bool:
    if start == target:
        return any(start in edge for edge in edges)

    adj: dict[object, set[object]] = {}
    for u, v in edges:
        adj.setdefault(u, set()).add(v)
        adj.setdefault(v, set()).add(u)

    if start not in adj or target not in adj:
        return False

    seen = {start}
    queue = deque([start])
    while queue:
        cur = queue.popleft()
        for nxt in adj[cur]:
            if nxt == target:
                return True
            if nxt not in seen:
                seen.add(nxt)
                queue.append(nxt)
    return False


def test_add_remove_connected():
    g = DynamicGraph()
    assert not g.connected("a", "b")
    assert g.add_edge("a", "b")
    assert g.connected("a", "b")
    assert not g.add_edge("a", "b")
    assert g.remove_edge("a", "b")
    assert not g.connected("a", "b")


def test_edges_nodes_roundtrip():
    g = DynamicGraph()
    g.add_edge("a", "b")
    g.add_edge("b", "c")
    assert set(g.nodes()) == {"a", "b", "c"}
    assert {tuple(sorted(edge)) for edge in g.edges()} == {("a", "b"), ("b", "c")}


def test_connectivity_oracle():
    g = DynamicGraph()
    edges: set[tuple[object, object]] = set()

    for u, v in [(1, 2), (2, 3), (3, 4)]:
        g.add_edge(u, v)
        edges.add((u, v))

    for u, v in [(1, 4), (1, 3), (2, 4)]:
        assert g.connected(u, v) == _has_path(edges, u, v)

    g.remove_edge(2, 3)
    edges.remove((2, 3))
    assert g.connected(1, 4) == _has_path(edges, 1, 4)
