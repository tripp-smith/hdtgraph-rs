from __future__ import annotations

from collections import deque
from typing import Iterable, Optional, Tuple

try:
    from ._rustdynconn import DynamicGraphPy as _DynamicGraphImpl
except Exception:  # pragma: no cover - fallback for pure-Python execution
    _DynamicGraphImpl = None


class _DynamicGraphFallback:
    def __init__(self) -> None:
        self._adj: dict[object, set[object]] = {}

    def add_node(self, node: object) -> None:
        self._adj.setdefault(node, set())

    def add_edge(self, u: object, v: object) -> bool:
        if u == v:
            return False
        self.add_node(u)
        self.add_node(v)
        if v in self._adj[u]:
            return False
        self._adj[u].add(v)
        self._adj[v].add(u)
        return True

    def remove_edge(self, u: object, v: object) -> bool:
        if u not in self._adj or v not in self._adj[u]:
            return False
        self._adj[u].remove(v)
        self._adj[v].remove(u)
        return True

    def has_edge(self, u: object, v: object) -> bool:
        return u in self._adj and v in self._adj[u]

    def connected(self, u: object, v: object) -> bool:
        if u == v:
            return u in self._adj
        if u not in self._adj or v not in self._adj:
            return False
        seen = {u}
        queue = deque([u])
        while queue:
            cur = queue.popleft()
            for nxt in self._adj[cur]:
                if nxt == v:
                    return True
                if nxt not in seen:
                    seen.add(nxt)
                    queue.append(nxt)
        return False

    def nodes(self) -> list[object]:
        return list(self._adj.keys())

    def edges(self) -> list[Tuple[object, object]]:
        seen: set[frozenset[object]] = set()
        edges: list[Tuple[object, object]] = []
        for u, nbrs in self._adj.items():
            for v in nbrs:
                key = frozenset((u, v))
                if key in seen:
                    continue
                seen.add(key)
                edges.append((u, v))
        return edges

    def to_networkx(self):
        try:
            import networkx as nx
        except ModuleNotFoundError as exc:  # pragma: no cover
            raise RuntimeError("networkx is required for to_networkx()") from exc
        g = nx.Graph()
        g.add_nodes_from(self.nodes())
        g.add_edges_from(self.edges())
        return g

    def update(
        self,
        edges_add: Optional[Iterable[Tuple[object, object]]] = None,
        edges_remove: Optional[Iterable[Tuple[object, object]]] = None,
    ) -> None:
        if edges_add is not None:
            for u, v in edges_add:
                self.add_edge(u, v)
        if edges_remove is not None:
            for u, v in edges_remove:
                self.remove_edge(u, v)

    @property
    def n(self) -> int:
        return len(self._adj)

    @property
    def m(self) -> int:
        return sum(len(neigh) for neigh in self._adj.values()) // 2

    @property
    def levels(self) -> int:
        return 1


class DynamicGraph:
    def __init__(self) -> None:
        impl = _DynamicGraphImpl if _DynamicGraphImpl is not None else _DynamicGraphFallback
        self._inner = impl()

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
