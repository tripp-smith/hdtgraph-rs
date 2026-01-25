import networkx as nx
from rustdynconn import DynamicGraph


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


def test_networkx_oracle():
    g = DynamicGraph()
    nxg = nx.Graph()
    edges = [(1, 2), (2, 3), (3, 4)]
    for u, v in edges:
        g.add_edge(u, v)
        nxg.add_edge(u, v)
    for u, v in [(1, 4), (1, 3), (2, 4)]:
        assert g.connected(u, v) == nx.has_path(nxg, u, v)
    g.remove_edge(2, 3)
    nxg.remove_edge(2, 3)
    assert g.connected(1, 4) == nx.has_path(nxg, 1, 4)
