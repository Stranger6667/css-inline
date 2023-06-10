class CSSInliner:
    def __init__(
        self,
        inline_style_tags: bool = True,
        remove_style_tags: bool = True,
        base_url: str | None = None,
        load_remote_stylesheets: bool = True,
        extra_css: str | None = None,
        preallocate_node_capacity: int | None = None,
    ) -> None: ...
    def inline(self, html: str) -> str: ...
    def inline_many(self, html: list[str]) -> list[str]: ...

def inline(
    html: str,
    inline_style_tags: bool = True,
    remove_style_tags: bool = True,
    base_url: str | None = None,
    load_remote_stylesheets: bool = True,
    extra_css: str | None = None,
    preallocate_node_capacity: int | None = None,
) -> str: ...

def inline_many(
    html: list[str],
    inline_style_tags: bool = True,
    remove_style_tags: bool = True,
    base_url: str | None = None,
    load_remote_stylesheets: bool = True,
    extra_css: str | None = None,
    preallocate_node_capacity: int | None = None,
) -> list[str]: ...

class InlineError(ValueError): ...
