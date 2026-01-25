import networkx as nx
from rustdynconn import DynamicGraph


def main():
    nxg = nx.Graph()
    nxg.add_edge("a", "b")
    nxg.add_edge("b", "c")

    g = DynamicGraph()
    for u, v in nxg.edges():
        g.add_edge(u, v)

    snapshot = g.to_networkx()
    print("Converted back edges:", list(snapshot.edges()))


if __name__ == "__main__":
    main()
