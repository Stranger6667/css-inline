class CSSInliner:
    def __init__(
        self,
        inline_style_tags: bool = True,
        keep_style_tags: bool = False,
        keep_link_tags: bool = False,
        base_url: str | None = None,
        load_remote_stylesheets: bool = True,
        extra_css: str | None = None,
        preallocate_node_capacity: int | None = 32,
    ) -> None: ...
    def inline(self, html: str) -> str: ...
    def inline_many(self, html: list[str]) -> list[str]: ...

def inline(
    html: str,
    inline_style_tags: bool = True,
    keep_style_tags: bool = False,
    keep_link_tags: bool = False,
    base_url: str | None = None,
    load_remote_stylesheets: bool = True,
    extra_css: str | None = None,
    preallocate_node_capacity: int | None = 32,
) -> str: ...
def inline_many(
    html: list[str],
    inline_style_tags: bool = True,
    keep_style_tags: bool = False,
    keep_link_tags: bool = False,
    base_url: str | None = None,
    load_remote_stylesheets: bool = True,
    extra_css: str | None = None,
    preallocate_node_capacity: int | None = 32,
) -> list[str]: ...

class InlineError(ValueError): ...
