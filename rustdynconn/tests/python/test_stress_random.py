import random

from rustdynconn import DynamicGraph


def test_random_ops():
    rng = random.Random(0)
    g = DynamicGraph()
    nodes = list(range(25))
    for _ in range(200):
        u, v = rng.sample(nodes, 2)
        if rng.random() < 0.5:
            g.add_edge(u, v)
        else:
            g.remove_edge(u, v)
        g.connected(u, v)
