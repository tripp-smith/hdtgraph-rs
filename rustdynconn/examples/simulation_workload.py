import random
import time

from rustdynconn import DynamicGraph


def main():
    rng = random.Random(0)
    g = DynamicGraph()
    nodes = list(range(200))
    start = time.time()
    ops = 2000
    for _ in range(ops):
        u, v = rng.sample(nodes, 2)
        if rng.random() < 0.6:
            g.add_edge(u, v)
        else:
            g.remove_edge(u, v)
        if rng.random() < 0.4:
            g.connected(u, v)
    elapsed = time.time() - start
    print(f"Completed {ops} ops in {elapsed:.4f}s")


if __name__ == "__main__":
    main()
