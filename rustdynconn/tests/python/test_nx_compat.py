from rustdynconn import DynamicGraphAdapter


def test_adapter_basic():
    g = DynamicGraphAdapter()
    assert g.add_edge("x", "y")
    assert g.has_edge("x", "y")
    assert g.connected("x", "y")
    g.remove_edge("x", "y")
    assert not g.has_edge("x", "y")
