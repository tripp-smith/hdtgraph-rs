import random
import networkx as nx
from rustdynconn import DynamicGraph


def test_connected_benchmark(benchmark):
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

    benchmark(run)
