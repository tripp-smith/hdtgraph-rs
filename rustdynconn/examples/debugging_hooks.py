from rustdynconn import DynamicGraph


def main():
    g = DynamicGraph()
    g.add_edge(1, 2)
    g.add_edge(2, 3)
    print("levels:", g.levels)
    print("nodes:", g.nodes())
    print("edges:", g.edges())


if __name__ == "__main__":
    main()
