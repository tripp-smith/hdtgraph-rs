from __future__ import annotations

from typing import Iterable, Optional, Tuple

try:
    from ._rustdynconn import DynamicGraphPy as _DynamicGraphImpl
except Exception:  # pragma: no cover - fallback for type checking only
    _DynamicGraphImpl = None


class DynamicGraph:
    def __init__(self) -> None:
        if _DynamicGraphImpl is None:
            raise RuntimeError("rustdynconn extension module not available")
        self._inner = _DynamicGraphImpl()

    def add_node(self, node: object) -> None:
        self._inner.add_node(node)

    def add_edge(self, u: object, v: object) -> bool:
        return self._inner.add_edge(u, v)

    def remove_edge(self, u: object, v: object) -> bool:
        return self._inner.remove_edge(u, v)

    def has_edge(self, u: object, v: object) -> bool:
        return self._inner.has_edge(u, v)

    def connected(self, u: object, v: object) -> bool:
        return self._inner.connected(u, v)

    def nodes(self) -> list[object]:
        return self._inner.nodes()

    def edges(self) -> list[Tuple[object, object]]:
        return self._inner.edges()

    def to_networkx(self):
        return self._inner.to_networkx()

    def update(
        self,
        edges_add: Optional[Iterable[Tuple[object, object]]] = None,
        edges_remove: Optional[Iterable[Tuple[object, object]]] = None,
    ) -> None:
        self._inner.update(edges_add, edges_remove)

    @property
    def n(self) -> int:
        return self._inner.n

    @property
    def m(self) -> int:
        return self._inner.m

    @property
    def levels(self) -> int:
        return self._inner.levels
