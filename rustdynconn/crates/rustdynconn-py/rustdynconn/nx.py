from __future__ import annotations

from typing import Iterable, Tuple

from ._typing import DynamicGraph


class DynamicGraphAdapter:
    """NetworkX-like adapter for DynamicGraph."""

    def __init__(self) -> None:
        self._graph = DynamicGraph()

    def add_edge(self, u: object, v: object) -> bool:
        return self._graph.add_edge(u, v)

    def remove_edge(self, u: object, v: object) -> bool:
        return self._graph.remove_edge(u, v)

    def has_edge(self, u: object, v: object) -> bool:
        return self._graph.has_edge(u, v)

    def nodes(self) -> list[object]:
        return self._graph.nodes()

    def edges(self) -> list[Tuple[object, object]]:
        return self._graph.edges()

    def connected(self, u: object, v: object) -> bool:
        return self._graph.connected(u, v)

    def update(
        self,
        edges_add: Iterable[Tuple[object, object]] | None = None,
        edges_remove: Iterable[Tuple[object, object]] | None = None,
    ) -> None:
        self._graph.update(edges_add, edges_remove)

    def to_networkx(self):
        return self._graph.to_networkx()
