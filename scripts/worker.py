#!/usr/bin/env python3
"""
Simple worker for flowz-mcp testing.
Reads WorkerRequest from stdin, outputs WorkerResponse to stdout.
"""

import json
import sys
import asyncio
from typing import Any

async def research_topic(topic: str, prompt: str) -> dict:
    """Simulate research by returning structured results."""
    
    # Since we can't actually do web research, return mock data based on topic
    results = {
        "rust-async": {
            "summary": "Rust async patterns: ใช้ async/await กับ Future trait, Pin/Unpin สำหรับ self-referential types, executors เช่น tokio/async-std จัดการ runtime, patterns รวมถึง select! join! spawn, error handling ด้วย Result/Option ใน async context",
            "sources": [
                "https://doc.rust-lang.org/book/ch17-00-async-await.html",
                "https://tokio.rs/tokio/tutorial",
                "https://rust-lang.github.io/async-book/"
            ]
        },
        "mcp-protocol": {
            "summary": "MCP (Model Context Protocol): JSON-RPC 2.0 transport, server/client roles, 3 primitives - tools/resources/prompts, initialization flow กับ capabilities negotiation, ใช้สำหรับเชื่อมต่อ LLM กับ external tools/data",
            "sources": [
                "https://modelcontextprotocol.io/",
                "https://github.com/modelcontextprotocol/specification",
                "https://docs.anthropic.com/en/docs/build-with-claude/mcp"
            ]
        },
        "subagent-delegation": {
            "summary": "Subagent delegation best practices: ใช้ worktree isolation, PTY สำหรับ interactive CLI, context passing ผ่าน prompt, result verification ด้วย schema, parallel สำหรับ independent tasks, pitfalls: auth, timeout, cleanup worktree",
            "sources": [
                "https://github.com/anthropic-ai/claude-code",
                "https://github.com/openai/codex",
                "https://github.com/opencode-ai/opencode"
            ]
        }
    }
    
    return results.get(topic, {
        "summary": f"Research completed for {topic}",
        "sources": []
    })

async def main():
    # Read request from stdin
    line = sys.stdin.readline()
    if not line:
        return
    
    try:
        request = json.loads(line.strip())
    except json.JSONDecodeError as e:
        response = {
            "item_id": "unknown",
            "ok": False,
            "error": {"code": "parse_error", "message": str(e)}
        }
        print(json.dumps(response))
        return
    
    item_id = request.get("item_id", "unknown")
    prompt = request.get("prompt", "")
    brief = request.get("brief", "")
    
    # Determine topic from brief
    brief_lower = brief.lower()
    if "rust" in brief_lower and "async" in brief_lower:
        topic = "rust-async"
    elif "mcp" in brief_lower or "model context" in brief_lower:
        topic = "mcp-protocol"
    elif "subagent" in brief_lower or "delegation" in brief_lower:
        topic = "subagent-delegation"
    else:
        topic = brief_lower.replace(" ", "-")
    
    # Do research
    result = await research_topic(topic, prompt)
    
    # Build response
    response = {
        "item_id": item_id,
        "ok": True,
        "value": result,
        "files": [],
        "error": None
    }
    
    print(json.dumps(response))

if __name__ == "__main__":
    asyncio.run(main())