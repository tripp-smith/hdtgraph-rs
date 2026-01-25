import networkx as nx
from rustdynconn import DynamicGraph


def main():
    g = DynamicGraph()
    nxg = nx.Graph()
    edges = [(i, i + 1) for i in range(10)]
    for u, v in edges:
        g.add_edge(u, v)
        nxg.add_edge(u, v)
    g.remove_edge(4, 5)
    nxg.remove_edge(4, 5)
    print("Dynamic connected 0->9:", g.connected(0, 9))
    print("NetworkX connected 0->9:", nx.has_path(nxg, 0, 9))


if __name__ == "__main__":
    main()
