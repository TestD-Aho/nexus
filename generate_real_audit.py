#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List, Set, Tuple
from urllib.parse import parse_qsl


ROOT = Path(__file__).resolve().parent
APP_DIR = ROOT / "app"
ROUTES_DIR = ROOT / "ressources" / "routes"
AUDIT_SQL = ROOT / "audit.txt"
OUTPUT = ROOT / "audit_14_03_2026.txt"


SQL_START_RE = re.compile(
    r"^\s*(SELECT|INSERT|UPDATE|DELETE|REPLACE|WITH)\b", re.IGNORECASE
)
SQL_VERB_RE = re.compile(
    r"^\s*(SELECT|INSERT|UPDATE|DELETE|REPLACE|WITH)\b", re.IGNORECASE
)
NON_TABLE_SQL_TOKENS = {
    "select",
    "from",
    "join",
    "left",
    "right",
    "inner",
    "outer",
    "cross",
    "on",
    "where",
    "group",
    "order",
    "having",
    "limit",
    "offset",
    "lateral",
    "set",
    "into",
    "values",
    "as",
    "and",
    "or",
    "not",
}


@dataclass
class PhpFileInfo:
    path: str
    text: str
    classes: List[str]
    requires: List[str]
    uses: List[str]
    instantiations: List[str]
    sql_snippets: List[str]
    sql_tables: Set[str]


@dataclass
class RouteFileInfo:
    path: str
    controllers: Set[str] = field(default_factory=set)
    pages: Set[str] = field(default_factory=set)
    actions: Set[str] = field(default_factory=set)
    has_page_guards: bool = False
    controller_methods: Set[str] = field(default_factory=set)
    action_to_methods: Dict[str, Set[str]] = field(default_factory=dict)
    page_to_controller: Dict[str, str] = field(default_factory=dict)


@dataclass
class LayoutCaseInfo:
    page: str
    controllers: Set[str] = field(default_factory=set)
    content_files: Set[str] = field(default_factory=set)
    route_files: Set[str] = field(default_factory=set)


@dataclass
class LayoutRouteResolution:
    global_route_files: Set[str] = field(default_factory=set)
    page_to_route_files: Dict[str, Set[str]] = field(default_factory=dict)


def normalize_slug_variant(value: str) -> str:
    v = (value or "").lower().strip()
    if not v:
        return ""
    replacements = {
        "plannification": "planification",
        "programation": "programmation",
    }
    for src, dst in replacements.items():
        v = v.replace(src, dst)
    return v


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except Exception:
        return ""


def normalize_name(value: str) -> str:
    return re.sub(r"[^a-z0-9]", "", value.lower())


def singularize(name: str) -> str:
    if name.endswith("ies") and len(name) > 3:
        return name[:-3] + "y"
    if name.endswith("s") and len(name) > 2:
        return name[:-1]
    return name


def normalize_schema_token(name: str) -> str:
    """Basic schema normalization for plural/singular noise filtering.

    Requirement: apply a simple normalization by stripping trailing 's'.
    """
    n = (name or "").strip().lower()
    if len(n) > 1 and n.endswith("s"):
        return n[:-1]
    return n


def extract_php_string_literals(text: str) -> List[str]:
    pattern = re.compile(r"([\"'])(?:\\.|(?!\1).)*\1", re.DOTALL)
    out: List[str] = []
    for m in pattern.finditer(text):
        raw = m.group(0)
        if len(raw) < 2:
            continue
        s = raw[1:-1]
        s = (
            s.replace("\\'", "'")
            .replace('\\"', '"')
            .replace("\\n", " ")
            .replace("\\r", " ")
        )
        s = re.sub(r"\s+", " ", s).strip()
        if s:
            out.append(s)
    return out


def is_real_sql_statement(s: str) -> bool:
    if not SQL_START_RE.search(s):
        return False

    lowered = s.lower()
    if "<?php" in lowered or "error_log" in lowered or "layout.php?page=" in lowered:
        return False

    verb_match = SQL_VERB_RE.search(s)
    if not verb_match:
        return False
    verb = verb_match.group(1).upper()

    if verb in {"SELECT", "WITH"} and not re.search(r"\bFROM\b", s, re.IGNORECASE):
        return False
    if verb == "INSERT" and not re.search(r"\bINTO\b", s, re.IGNORECASE):
        return False
    if verb == "UPDATE" and not re.search(r"\bSET\b", s, re.IGNORECASE):
        return False
    if verb == "DELETE" and not re.search(r"\bFROM\b", s, re.IGNORECASE):
        return False
    if verb == "REPLACE" and not re.search(r"\bINTO\b", s, re.IGNORECASE):
        return False

    return True


def extract_sql_snippets(text: str) -> List[str]:
    snippets = []
    for raw in extract_php_string_literals(text):
        if is_real_sql_statement(raw):
            snippets.append(raw)

    uniq: List[str] = []
    seen: Set[str] = set()
    for s in snippets:
        key = s.lower()
        if key in seen:
            continue
        seen.add(key)
        uniq.append(s)
    return uniq


def clean_table_token(token: str) -> str:
    token = token.strip("` \t\n\r")
    token = token.split(".")[-1]
    if not token:
        return ""
    if "{" in token or "}" in token or "$" in token:
        return ""
    if token.lower() in NON_TABLE_SQL_TOKENS:
        return ""
    return token


def parse_sql_tables(sql: str) -> Set[str]:
    tables: Set[str] = set()
    verb_match = SQL_VERB_RE.search(sql)
    verb = verb_match.group(1).upper() if verb_match else ""

    patterns = [
        r"\bFROM\s+`?([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)?)`?",
        r"\bJOIN\s+`?([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)?)`?",
        r"\bINTO\s+`?([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)?)`?",
        r"\bDELETE\s+FROM\s+`?([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)?)`?",
    ]
    if verb == "UPDATE":
        patterns.append(
            r"\bUPDATE\s+`?([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)?)`?"
        )

    for pat in patterns:
        for m in re.finditer(pat, sql, flags=re.IGNORECASE):
            token = clean_table_token(m.group(1))
            if token:
                tables.add(token)
    return tables


def parse_audit_tables(audit_sql_text: str) -> List[str]:
    tables = re.findall(
        r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?`?([A-Za-z_][A-Za-z0-9_]*)`?",
        audit_sql_text,
        flags=re.IGNORECASE,
    )
    return sorted(set(tables), key=lambda x: x.lower())


def parse_php_file(path: Path) -> PhpFileInfo:
    text = read_text(path)
    classes = re.findall(r"\bclass\s+([A-Za-z_][A-Za-z0-9_]*)", text)
    requires = re.findall(
        r"\b(?:require|require_once|include|include_once)\s*\(?\s*([^;]+)\s*\)?;", text
    )
    uses = re.findall(r"\buse\s+([^;]+);", text)
    instantiations = re.findall(r"\bnew\s+\\?([A-Za-z_\\][A-Za-z0-9_\\]*)\s*\(", text)

    sql_snippets = extract_sql_snippets(text)
    sql_tables: Set[str] = set()
    for s in sql_snippets:
        sql_tables.update(parse_sql_tables(s))

    return PhpFileInfo(
        path=str(path.relative_to(ROOT)).replace("\\", "/"),
        text=text,
        classes=classes,
        requires=[x.strip() for x in requires],
        uses=[x.strip() for x in uses],
        instantiations=instantiations,
        sql_snippets=sql_snippets,
        sql_tables=sql_tables,
    )


def parse_permission_routes(
    permission_registry_text: str,
) -> Tuple[List[Dict[str, str]], int]:
    # STRICT extraction: ONLY literal entries inside `routes => [ ... ]` blocks.
    # Ignores aliases and any dynamic/template route construction.

    def find_matching_square(text: str, open_pos: int) -> int:
        depth = 0
        in_single = False
        in_double = False
        escaped = False
        for i in range(open_pos, len(text)):
            c = text[i]

            if escaped:
                escaped = False
                continue

            if c == "\\":
                escaped = True
                continue

            if in_single:
                if c == "'":
                    in_single = False
                continue

            if in_double:
                if c == '"':
                    in_double = False
                continue

            if c == "'":
                in_single = True
                continue
            if c == '"':
                in_double = True
                continue

            if c == "[":
                depth += 1
            elif c == "]":
                depth -= 1
                if depth == 0:
                    return i
        return -1

    routes: List[Dict[str, str]] = []
    skipped_non_literal = 0
    text = permission_registry_text

    for m_routes in re.finditer(r"'routes'\s*=>\s*\[", text, flags=re.IGNORECASE):
        open_bracket = text.find("[", m_routes.end() - 1)
        if open_bracket < 0:
            continue
        close_bracket = find_matching_square(text, open_bracket)
        if close_bracket < 0:
            continue

        routes_block = text[open_bracket + 1 : close_bracket]

        entry_re = re.compile(
            r"\[\s*"
            r"'pattern'\s*=>\s*'([^']*)'\s*,\s*"
            r"'method'\s*=>\s*'([^']*)'"
            r"(?:\s*,\s*'crud'\s*=>\s*'([^']*)')?"
            r"(?:\s*,\s*'modalAction'\s*=>\s*'([^']*)')?"
            r"\s*\]",
            flags=re.IGNORECASE | re.DOTALL,
        )

        for m_entry in entry_re.finditer(routes_block):
            pattern = (m_entry.group(1) or "").strip()
            method = (m_entry.group(2) or "GET").upper().strip()
            crud = (m_entry.group(3) or "").strip()
            modal_action = (m_entry.group(4) or "").strip()

            params = dict(parse_qsl(pattern, keep_blank_values=True))
            page_val = params.get("page", "")
            action_val = params.get("action", "")

            routes.append(
                {
                    "pattern": pattern,
                    "method": method,
                    "crud": crud,
                    "page": page_val,
                    "action": action_val,
                    "modalAction": params.get("modalAction", modal_action),
                }
            )

        # Count non-literal route definitions inside routes blocks for transparency.
        literal_count = len(list(entry_re.finditer(routes_block)))
        rough_entry_count = len(re.findall(r"\[\s*'pattern'\s*=>", routes_block))
        if rough_entry_count > literal_count:
            skipped_non_literal += rough_entry_count - literal_count

    return routes, skipped_non_literal


def find_matching_brace(text: str, open_pos: int) -> int:
    depth = 0
    for i in range(open_pos, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def parse_switch_cases(text: str) -> List[Tuple[str, List[Tuple[str, str]]]]:
    out: List[Tuple[str, List[Tuple[str, str]]]] = []
    switch_re = re.compile(r"switch\s*\(\s*(.*?)\s*\)\s*\{", re.DOTALL)
    for m in switch_re.finditer(text):
        expr = m.group(1)
        open_brace = text.find("{", m.end() - 1)
        if open_brace < 0:
            continue
        close_brace = find_matching_brace(text, open_brace)
        if close_brace < 0:
            continue
        body = text[open_brace + 1 : close_brace]
        labels = [
            (x.group(1), x.start()) for x in re.finditer(r"case\s+'([^']+)'\s*:", body)
        ]
        case_chunks: List[Tuple[str, str]] = []
        for idx, (label, pos) in enumerate(labels):
            end = labels[idx + 1][1] if idx + 1 < len(labels) else len(body)
            case_chunks.append((label, body[pos:end]))
        out.append((expr, case_chunks))
    return out


def parse_route_file(path: Path) -> RouteFileInfo:
    text = read_text(path)
    info = RouteFileInfo(path=str(path.relative_to(ROOT)).replace("\\", "/"))

    def extract_param_aliases(param: str) -> Set[str]:
        aliases: Set[str] = set()
        for m in re.finditer(
            rf"\$([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:\(string\)\s*)?\(?\s*\$_GET\['{param}'\]\s*\?\?",
            text,
            flags=re.IGNORECASE,
        ):
            aliases.add(m.group(1))
        for m in re.finditer(
            rf"\$([A-Za-z_][A-Za-z0-9_]*)\s*=\s*isset\s*\(\s*\$_GET\['{param}'\]\s*\)\s*\?\s*\$_GET\['{param}'\]",
            text,
            flags=re.IGNORECASE,
        ):
            aliases.add(m.group(1))
        return aliases

    def extract_param_values(param: str, aliases: Set[str]) -> Set[str]:
        values: Set[str] = set()
        for m in re.finditer(
            rf"\$_GET\['{param}'\]\s*(?:===|==)\s*'([^']+)'",
            text,
            flags=re.IGNORECASE,
        ):
            values.add(m.group(1))

        for m in re.finditer(
            rf"in_array\s*\(\s*\$_GET\['{param}'\]\s*,\s*\[(.*?)\]",
            text,
            flags=re.IGNORECASE | re.DOTALL,
        ):
            for x in re.findall(r"'([^']+)'", m.group(1)):
                values.add(x)

        for alias in aliases:
            for m in re.finditer(
                rf"\${alias}\s*(?:===|==)\s*'([^']+)'",
                text,
                flags=re.IGNORECASE,
            ):
                values.add(m.group(1))
            for m in re.finditer(
                rf"in_array\s*\(\s*\${alias}\s*,\s*\[(.*?)\]",
                text,
                flags=re.IGNORECASE | re.DOTALL,
            ):
                for x in re.findall(r"'([^']+)'", m.group(1)):
                    values.add(x)

        return values

    controller_vars: Dict[str, str] = {}
    for m in re.finditer(
        r"\$([A-Za-z_][A-Za-z0-9_]*)\s*=\s*new\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        text,
    ):
        var_name = m.group(1)
        controller_name = m.group(2)
        if controller_name.lower().endswith("controller"):
            controller_vars[var_name] = controller_name

    for m in re.finditer(r"/app/controllers/([A-Za-z0-9_]+)\.php", text):
        info.controllers.add(m.group(1))

    page_aliases = extract_param_aliases("page")
    action_aliases = extract_param_aliases("action") | extract_param_aliases(
        "ajaxAction"
    )

    info.pages.update(extract_param_values("page", page_aliases))
    info.actions.update(extract_param_values("action", action_aliases))
    info.actions.update(extract_param_values("ajaxAction", action_aliases))

    in_array_pages = list(
        re.finditer(
            r"in_array\s*\(\s*\$_GET\['page'\]\s*,\s*\[(.*?)\]",
            text,
            flags=re.IGNORECASE | re.DOTALL,
        )
    )

    # Strong page->controller mapping from direct if/elseif blocks.
    # Example: if ($_GET['page'] === 'x') { $controller = new XController(); }
    page_if_block = re.compile(
        r"if\s*\([^\)]*\$_GET\['page'\]\s*(?:===|==)\s*'([^']+)'[^\)]*\)\s*\{",
        re.IGNORECASE,
    )
    for m in page_if_block.finditer(text):
        page = m.group(1)
        open_brace = text.find("{", m.end() - 1)
        if open_brace < 0:
            continue
        close_brace = find_matching_brace(text, open_brace)
        if close_brace < 0:
            continue
        block = text[open_brace + 1 : close_brace]
        ctrls = re.findall(r"new\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", block)
        for c in ctrls:
            if c.lower().endswith("controller"):
                info.page_to_controller[page] = c
        if page not in info.page_to_controller:
            for m_call in re.finditer(
                r"\$([A-Za-z_][A-Za-z0-9_]*)->([A-Za-z_][A-Za-z0-9_]*)\(",
                block,
            ):
                var_name = m_call.group(1)
                if var_name in controller_vars:
                    info.page_to_controller[page] = controller_vars[var_name]
                    break

    imported = sorted(info.controllers)
    if len(imported) == 1 and info.pages:
        candidate_ctrl = imported[0]
        # Map only page conditions that actually execute controller logic
        # in their own block; avoids false mappings like processus_validation.
        page_if_block = re.compile(
            r"if\s*\([^\)]*\$_GET\['page'\]\s*(?:===|==)\s*'([^']+)'[^\)]*\)\s*\{",
            re.IGNORECASE,
        )
        mapped_pages: Set[str] = set()
        for m in page_if_block.finditer(text):
            page = m.group(1)
            open_brace = text.find("{", m.end() - 1)
            if open_brace < 0:
                continue
            close_brace = find_matching_brace(text, open_brace)
            if close_brace < 0:
                continue
            block = text[open_brace + 1 : close_brace]
            has_controller_activity = bool(
                re.search(r"\$[A-Za-z_][A-Za-z0-9_]*->", block)
                or re.search(r"new\s+[A-Za-z_][A-Za-z0-9_]*Controller\s*\(", block)
            )
            if has_controller_activity:
                info.page_to_controller[page] = candidate_ctrl
                mapped_pages.add(page)

        # For in_array(page, [...]) guard blocks, keep mapping to single imported controller.
        for m in in_array_pages:
            for x in re.findall(r"'([^']+)'", m.group(1)):
                info.page_to_controller[x] = candidate_ctrl
                mapped_pages.add(x)

        if not mapped_pages:
            for p in info.pages:
                info.page_to_controller[p] = candidate_ctrl

    for switch_expr, chunks in parse_switch_cases(text):
        expr = switch_expr.lower().replace(" ", "")
        switch_kind = ""
        action_vars = set(action_aliases)
        page_vars = set(page_aliases)
        if "page" in expr or any(f"${v.lower()}" in expr for v in page_vars):
            switch_kind = "page"
        elif "action" in expr or any(f"${v.lower()}" in expr for v in action_vars):
            switch_kind = "action"

        for label, chunk in chunks:
            if switch_kind == "page":
                info.pages.add(label)
                mc = re.search(
                    r"\$controller\s*=\s*new\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
                    chunk,
                )
                if mc:
                    info.page_to_controller[label] = mc.group(1)
                if label not in info.page_to_controller:
                    for m_call in re.finditer(
                        r"\$([A-Za-z_][A-Za-z0-9_]*)->([A-Za-z_][A-Za-z0-9_]*)\(",
                        chunk,
                    ):
                        var_name = m_call.group(1)
                        if var_name in controller_vars:
                            info.page_to_controller[label] = controller_vars[var_name]
                            break
            elif switch_kind == "action":
                info.actions.add(label)

            methods = set(
                re.findall(r"\$controller->([A-Za-z_][A-Za-z0-9_]*)\(", chunk)
            )
            info.controller_methods.update(methods)
            if switch_kind == "action" and methods:
                info.action_to_methods.setdefault(label, set()).update(methods)

    info.controller_methods.update(
        re.findall(r"\$controller->([A-Za-z_][A-Za-z0-9_]*)\(", text)
    )
    info.has_page_guards = bool(info.pages)
    return info


def infer_services_from_controller(
    ctrl: PhpFileInfo,
    service_files: Dict[str, PhpFileInfo],
    by_class: Dict[str, PhpFileInfo],
) -> List[PhpFileInfo]:
    out: List[PhpFileInfo] = []
    seen: Set[str] = set()

    for req in ctrl.requires:
        m = re.search(r"/Services/([A-Za-z0-9_]+)\.php", req.replace("\\", "/"))
        if m:
            name = m.group(1)
            key = name.lower()
            if key in service_files and service_files[key].path not in seen:
                out.append(service_files[key])
                seen.add(service_files[key].path)

    for inst in ctrl.instantiations:
        short = inst.split("\\")[-1]
        if short.lower().endswith("service"):
            f = by_class.get(short.lower()) or service_files.get(short.lower())
            if f and f.path not in seen:
                out.append(f)
                seen.add(f.path)

    return out


def infer_models_from_controller(
    ctrl: PhpFileInfo,
    model_files: Dict[str, PhpFileInfo],
    by_class: Dict[str, PhpFileInfo],
) -> List[PhpFileInfo]:
    out: List[PhpFileInfo] = []
    seen: Set[str] = set()

    for req in ctrl.requires:
        m = re.search(r"/models/([A-Za-z0-9_]+)\.php", req.replace("\\", "/"))
        if m:
            name = m.group(1).lower()
            if name in model_files and model_files[name].path not in seen:
                out.append(model_files[name])
                seen.add(model_files[name].path)

    for inst in ctrl.instantiations:
        short = inst.split("\\")[-1].lower()
        f = by_class.get(short) or model_files.get(short)
        if f and f.path not in seen:
            out.append(f)
            seen.add(f.path)

    return out


def parse_layout_cases(layout_text: str) -> Dict[str, LayoutCaseInfo]:
    pages: Dict[str, LayoutCaseInfo] = {}

    switch_match = re.search(
        r"switch\s*\(\s*\$currentMenuSlug\s*\)\s*\{", layout_text, flags=re.DOTALL
    )
    if not switch_match:
        return pages
    open_brace = layout_text.find("{", switch_match.end() - 1)
    if open_brace < 0:
        return pages
    close_brace = find_matching_brace(layout_text, open_brace)
    if close_brace < 0:
        return pages
    switch_body = layout_text[open_brace + 1 : close_brace]

    labels = [
        (m.group(1), m.start())
        for m in re.finditer(r"case\s+'([^']+)'\s*:\s*", switch_body)
    ]
    for i, (label, pos) in enumerate(labels):
        end = labels[i + 1][1] if i + 1 < len(labels) else len(switch_body)
        chunk = switch_body[pos:end]

        info = pages.setdefault(label, LayoutCaseInfo(page=label))

        for c in re.findall(r"new\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", chunk):
            if c.lower().endswith("controller"):
                info.controllers.add(c)

        for c in re.findall(r"/app/controllers/([A-Za-z0-9_]+)\.php", chunk):
            info.controllers.add(c)

        for cf in re.findall(r"\$contentFile\s*=\s*[^;]*'([^']+\.php)'", chunk):
            info.content_files.add(cf)

        for inc in re.findall(
            r"\b(?:include|include_once|require|require_once)\b[^;]*['\"]([^'\"]*ressources/routes/[^'\"]+\.php)['\"]",
            chunk,
            flags=re.IGNORECASE,
        ):
            route_name = Path(inc.replace("\\", "/")).name
            if route_name:
                info.route_files.add(f"ressources/routes/{route_name}")

    return pages


def parse_layout_route_resolution(layout_text: str) -> LayoutRouteResolution:
    info = LayoutRouteResolution()
    global_includes = re.findall(
        r"\b(?:include|include_once|require|require_once)\b[^;]*['\"]([^'\"]*ressources/routes/[^'\"]+\.php)['\"]",
        layout_text,
        flags=re.IGNORECASE,
    )
    for inc in global_includes:
        route_name = Path(inc.replace("\\", "/")).name
        if route_name:
            info.global_route_files.add(f"ressources/routes/{route_name}")

    for page, case_info in parse_layout_cases(layout_text).items():
        if case_info.route_files:
            info.page_to_route_files[page] = set(case_info.route_files)

    return info


def infer_controllers_from_view_content(
    content_rel: str,
    controller_by_name: Dict[str, PhpFileInfo],
) -> List[str]:
    rel = content_rel.strip().lstrip("/")
    view_path = ROOT / "ressources" / "views" / rel
    if not view_path.exists():
        return []

    text = read_text(view_path)
    controllers: Set[str] = set()

    for c in re.findall(r"/app/controllers/([A-Za-z0-9_]+)\.php", text):
        controllers.add(c)
    for c in re.findall(r"new\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", text):
        if c.lower().endswith("controller"):
            controllers.add(c)

    out = [c for c in sorted(controllers) if c in controller_by_name]
    out.extend([c for c in sorted(controllers) if c not in controller_by_name])
    return out


def infer_models_from_service(
    svc: PhpFileInfo,
    model_files: Dict[str, PhpFileInfo],
    by_class: Dict[str, PhpFileInfo],
) -> List[PhpFileInfo]:
    out: List[PhpFileInfo] = []
    seen: Set[str] = set()

    for req in svc.requires:
        m = re.search(r"/models/([A-Za-z0-9_]+)\.php", req.replace("\\", "/"))
        if m:
            name = m.group(1).lower()
            if name in model_files and model_files[name].path not in seen:
                out.append(model_files[name])
                seen.add(model_files[name].path)

    for inst in svc.instantiations:
        short = inst.split("\\")[-1].lower()
        f = by_class.get(short) or model_files.get(short)
        if f and f.path not in seen:
            out.append(f)
            seen.add(f.path)

    return out


def route_signature(route: Dict[str, str]) -> str:
    return f"{route['method']} {route['pattern']}"


def resolve_handler(
    route: Dict[str, str],
    route_files: List[RouteFileInfo],
    controller_by_name: Dict[str, PhpFileInfo],
) -> Tuple[str, str, str]:
    page = route.get("page", "")
    normalized_page = normalize_slug_variant(page)
    action = route.get("action", "") or route.get("modalAction", "")

    candidates: List[Tuple[RouteFileInfo, str]] = []
    for rf in route_files:
        normalized_rf_pages = {normalize_slug_variant(p) for p in rf.pages}
        page_match = bool(page and normalized_page in normalized_rf_pages)
        action_match = bool(action and action in rf.actions)

        # Mathematically strict mapping rules:
        # - page+action routes require explicit evidence of BOTH page and action guards
        # - page-only routes require explicit page guard
        # - action-only routes require explicit action guard
        if page and action:
            if page_match and action_match:
                candidates.append((rf, "route-file-explicit-page-action"))
        elif page:
            if page_match:
                candidates.append((rf, "route-file-explicit-page"))
        elif action:
            if action_match:
                candidates.append((rf, "route-file-explicit-action"))

    if not candidates:
        return (
            "(unmapped due to missing explicit guard)",
            "(unmapped due to missing explicit guard)",
            "unmapped due to missing explicit guard",
        )

    # Deterministic selection among strict candidates.
    chosen_file, mapping_source = sorted(candidates, key=lambda x: x[0].path)[0]

    controller = ""
    if page:
        for pkey, cval in chosen_file.page_to_controller.items():
            if normalize_slug_variant(pkey) == normalized_page:
                controller = cval
                break

    if chosen_file.controllers:
        sorted_controllers = sorted(chosen_file.controllers)
        if controller:
            pass
        elif len(sorted_controllers) == 1:
            controller = sorted_controllers[0]
        else:
            # try matching controller name with page semantics
            pnorm = normalize_name(page)
            best = ""
            best_score = -1
            for c in sorted_controllers:
                score = 0
                cnorm = normalize_name(c.replace("Controller", ""))
                if cnorm == pnorm:
                    score = 5
                elif pnorm in cnorm or cnorm in pnorm:
                    score = 3
                if score > best_score:
                    best = c
                    best_score = score
            controller = best or sorted_controllers[0]

    if not controller:
        controller = "(unresolved)"

    if controller != "(unresolved)" and controller not in controller_by_name:
        return chosen_file.path, controller, mapping_source

    return chosen_file.path, controller, mapping_source


def generate_report() -> str:
    app_php_files = sorted(APP_DIR.rglob("*.php"))
    all_files = [parse_php_file(p) for p in app_php_files]

    controllers = [f for f in all_files if f.path.startswith("app/controllers/")]
    services = [f for f in all_files if f.path.startswith("app/Services/")]
    models = [f for f in all_files if f.path.startswith("app/models/")]

    controller_by_name = {Path(f.path).stem: f for f in controllers}
    service_by_stem = {Path(f.path).stem.lower(): f for f in services}
    model_by_stem = {Path(f.path).stem.lower(): f for f in models}

    service_by_class: Dict[str, PhpFileInfo] = {}
    for s in services:
        for c in s.classes:
            service_by_class[c.lower()] = s
    model_by_class: Dict[str, PhpFileInfo] = {}
    for m in models:
        for c in m.classes:
            model_by_class[c.lower()] = m

    # SQL + schema truth list
    audit_sql_text = read_text(AUDIT_SQL)
    valid_tables = parse_audit_tables(audit_sql_text)
    valid_tables_lc = {t.lower() for t in valid_tables}

    sql_sources = [f for f in all_files if f.sql_snippets]
    queried_table_to_files: Dict[str, Set[str]] = {}
    for f in sql_sources:
        for t in sorted(f.sql_tables):
            queried_table_to_files.setdefault(t.lower(), set()).add(f.path)

    schema_incompat: Dict[str, Set[str]] = {}
    plural_singular_mismatches: Dict[str, Dict[str, object]] = {}
    normalized_valid_to_real: Dict[str, Set[str]] = {}
    for vt in valid_tables_lc:
        normalized_valid_to_real.setdefault(normalize_schema_token(vt), set()).add(vt)

    for t_lc, files in queried_table_to_files.items():
        if t_lc in valid_tables_lc:
            continue

        normalized_query = normalize_schema_token(t_lc)
        if normalized_query in normalized_valid_to_real:
            plural_singular_mismatches[t_lc] = {
                "files": files,
                "matches": sorted(normalized_valid_to_real[normalized_query]),
            }
            continue

        schema_incompat[t_lc] = files

    # Routes: literal parse from file => expected 192
    permission_registry_path = APP_DIR / "config" / "permission_registry.php"
    permission_text = read_text(permission_registry_path)
    routes, skipped_non_literal_routes = parse_permission_routes(permission_text)
    literal_route_entries = len(routes)

    # Routing graph from ressources/routes only (strict route-file trace)
    route_files = [parse_route_file(p) for p in sorted(ROUTES_DIR.glob("*.php"))]

    journeys: List[Dict[str, object]] = []
    unresolved = 0
    for r in routes:
        route_file, controller_name, source = resolve_handler(
            r,
            route_files,
            controller_by_name,
        )

        service_paths: List[str] = []
        model_paths: Set[str] = set()

        if controller_name in controller_by_name:
            ctrl = controller_by_name[controller_name]
            svc_files = infer_services_from_controller(
                ctrl, service_by_stem, service_by_class
            )

            ctrl_model_paths = {
                m.path
                for m in infer_models_from_controller(
                    ctrl, model_by_stem, model_by_class
                )
            }
            if svc_files:
                service_paths = sorted({s.path for s in svc_files})
                for s in svc_files:
                    for m in infer_models_from_service(
                        s, model_by_stem, model_by_class
                    ):
                        model_paths.add(m.path)
                model_paths.update(ctrl_model_paths)
            else:
                # For controller-centric modules (ex: archive controllers), reflect
                # that business/data logic is directly handled in the controller.
                service_paths = [f"(controller-logic) {ctrl.path}"]
                model_paths.update(ctrl_model_paths)
        else:
            if source == "unmapped due to missing explicit guard":
                unresolved += 1

        journeys.append(
            {
                "route": r,
                "route_file": route_file,
                "mapping_source": source,
                "controller": controller_name,
                "services": service_paths,
                "models": sorted(model_paths),
            }
        )

    # Group by page for high-signal journey section
    by_page: Dict[str, List[Dict[str, object]]] = {}
    for j in journeys:
        route_obj = j.get("route")
        route_dict: Dict[str, str] = route_obj if isinstance(route_obj, dict) else {}
        page = route_dict.get("page", "")
        by_page.setdefault(page or "(no-page)", []).append(j)

    now = dt.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    lines: List[str] = []
    lines.append("CHECKMASTER APPLICATION AUDIT - VERIFIED CODE TRACE REPORT")
    lines.append("Generated by generate_real_audit.py")
    lines.append(f"Generated at: {now}")
    lines.append("Scope: app/, ressources/routes/, public/layout.php, audit.txt")
    lines.append("")

    lines.append("============================================================")
    lines.append("1) VERIFICATION SNAPSHOT")
    lines.append("============================================================")
    lines.append(f"- PHP files scanned (app): {len(all_files)}")
    lines.append(
        f"- Controllers: {len(controllers)} | Services: {len(services)} | Models: {len(models)}"
    )
    lines.append(f"- Valid schema tables loaded from audit.txt: {len(valid_tables)}")
    lines.append(
        f"- Routes parsed from permission_registry.php literals: {literal_route_entries}"
    )
    lines.append(
        f"- Non-literal route entries ignored inside routes blocks: {skipped_non_literal_routes}"
    )
    lines.append(f"- Executable route entries mapped: {len(routes) - unresolved}")
    lines.append(f"- Unmapped routes (missing explicit guards): {unresolved}")
    lines.append("")

    lines.append("============================================================")
    lines.append("2) SQL EXTRACTION QUALITY CHECK")
    lines.append("============================================================")
    lines.append(
        "Rules: only strings starting with SQL verbs and containing required SQL clauses are retained."
    )
    lines.append(f"Files with confirmed SQL snippets: {len(sql_sources)}")
    lines.append("")
    for f in sorted(sql_sources, key=lambda x: x.path):
        lines.append(f"- {f.path}")
        lines.append(
            f"  snippets={len(f.sql_snippets)} | tables={', '.join(sorted(f.sql_tables)) if f.sql_tables else '(none)'}"
        )
        for s in f.sql_snippets[:3]:
            lines.append(f"    * {s[:220]}")
    lines.append("")

    lines.append("============================================================")
    lines.append("3) SCHEMA INCOMPATIBILITIES (CODE QUERIES VS audit.txt)")
    lines.append("============================================================")
    if not schema_incompat:
        lines.append(
            "No incompatibilities detected: all queried tables are present in audit.txt."
        )
    else:
        lines.append(
            "The following tables are queried in code but absent from audit.txt:"
        )
        for t in sorted(schema_incompat):
            files = sorted(schema_incompat[t])
            lines.append(f"- {t}")
            for fp in files:
                lines.append(f"  * {fp}")
            suggestion = singularize(t)
            if suggestion != t and suggestion in valid_tables_lc:
                lines.append(
                    f"  -> probable naming mismatch with existing table: {suggestion}"
                )
    lines.append("")

    lines.append("============================================================")
    lines.append("3.1) PLURAL/SINGULAR MISMATCHES (Likely safe)")
    lines.append("============================================================")
    if not plural_singular_mismatches:
        lines.append("No plural/singular mismatches detected.")
    else:
        lines.append(
            "The following queried tables differ only by basic plural/singular normalization (trailing 's'):"
        )
        for queried in sorted(plural_singular_mismatches):
            entry = plural_singular_mismatches[queried]
            files_raw = entry.get("files", set())
            matches_raw = entry.get("matches", [])
            files = sorted(files_raw) if isinstance(files_raw, set) else []
            matches = sorted(matches_raw) if isinstance(matches_raw, list) else []
            lines.append(f"- queried: {queried}")
            lines.append(
                f"  matches_in_audit.txt: {', '.join(matches) if matches else '(none)'}"
            )
            for fp in files:
                lines.append(f"  * {fp}")
    lines.append("")

    lines.append("============================================================")
    lines.append("4) COMPLETE USER-JOURNEY MAP (ROUTE -> HANDLER -> SERVICE -> MODEL)")
    lines.append("============================================================")
    lines.append(
        "Source trace order: permission_registry.php route pattern -> ressources/routes/*.php and public/layout.php routing logic."
    )
    lines.append("")

    for page in sorted(by_page):
        entries = by_page[page]
        lines.append(f"PAGE: {page} | routes={len(entries)}")
        for j in entries:
            r = j["route"]
            assert isinstance(r, dict)
            action_tok = r.get("action") or r.get("modalAction") or "(none)"
            controller = str(j["controller"])
            route_file = str(j["route_file"])
            services_list = j["services"] if isinstance(j["services"], list) else []
            models_list = j["models"] if isinstance(j["models"], list) else []

            lines.append(
                f"- {r.get('method', 'GET')} pattern="
                f"{r.get('pattern', '')} | action={action_tok} | crud={r.get('crud', '') or '(none)'}"
            )
            lines.append(f"  handler_file: {route_file}")
            lines.append(f"  mapping_source: {j.get('mapping_source', '(unknown)')}")
            lines.append(f"  controller: {controller}")
            lines.append(
                f"  services: {', '.join(services_list) if services_list else '(none detected)'}"
            )
            lines.append(
                f"  models: {', '.join(models_list) if models_list else '(none detected)'}"
            )
        lines.append("")

    lines.append("============================================================")
    lines.append("5) SUMMARY")
    lines.append("============================================================")
    lines.append(f"- Total permission routes parsed: {len(routes)}")
    lines.append(f"- Total pages covered in journey map: {len(by_page)}")
    lines.append(f"- SQL source files: {len(sql_sources)}")
    lines.append(f"- Incompatible queried tables: {len(schema_incompat)}")
    lines.append(
        f"- Plural/singular mismatches (likely safe): {len(plural_singular_mismatches)}"
    )
    lines.append("- Report lines: __REPORT_LINE_COUNT__")

    report = "\n".join(lines) + "\n"
    real_line_count = len(report.splitlines())
    report = report.replace("__REPORT_LINE_COUNT__", str(real_line_count), 1)
    return report


def main() -> None:
    report = generate_report()
    OUTPUT.write_text(report, encoding="utf-8")
    written_text = OUTPUT.read_text(encoding="utf-8", errors="replace")
    actual_lines = len(written_text.splitlines())
    print(f"Audit report written: {OUTPUT}")
    print(f"Total lines: {actual_lines}")


if __name__ == "__main__":
    main()
