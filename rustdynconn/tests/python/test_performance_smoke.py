import random

import pytest
from _pytest.fixtures import FixtureLookupError

pytestmark = [pytest.mark.networkx, pytest.mark.benchmark]

from rustdynconn import DynamicGraph


def test_connected_benchmark(request):
    nx = pytest.importorskip("networkx")
    rng = random.Random(0)
    g = DynamicGraph()
    nxg = nx.Graph()
    edges = [(rng.randint(0, 200), rng.randint(0, 200)) for _ in range(400)]
    for u, v in edges:
        g.add_edge(u, v)
        nxg.add_edge(u, v)
    pairs = [(rng.randint(0, 200), rng.randint(0, 200)) for _ in range(200)]

    def run():
        for u, v in pairs:
            g.connected(u, v)

    try:
        benchmark = request.getfixturevalue("benchmark")
    except FixtureLookupError:
        run()
    else:
        benchmark(run)
