from __future__ import annotations

from ._typing import DynamicGraph


class DynamicGraphAdapter:
    def __init__(self) -> None:
        self.graph = DynamicGraph()

    def add_edge(self, u: object, v: object) -> bool:
        return self.graph.add_edge(u, v)

    def remove_edge(self, u: object, v: object) -> bool:
        return self.graph.remove_edge(u, v)

    def has_edge(self, u: object, v: object) -> bool:
        return self.graph.has_edge(u, v)

    def connected(self, u: object, v: object) -> bool:
        return self.graph.connected(u, v)

    def to_networkx(self):
        return self.graph.to_networkx()
