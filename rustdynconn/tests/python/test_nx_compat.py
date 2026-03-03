import pytest

from rustdynconn import DynamicGraphAdapter

pytestmark = pytest.mark.core


def test_adapter_basic():
    g = DynamicGraphAdapter()
    assert g.add_edge("x", "y")
    assert g.has_edge("x", "y")
    assert g.connected("x", "y")
    g.remove_edge("x", "y")
    assert not g.has_edge("x", "y")
