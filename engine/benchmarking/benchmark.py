#!/usr/bin/env python3
"""
Benchmarking Suite for WASM Search Engine

Tests:
1. Speed tests - measure search latency across different query types
2. Accuracy tests - verify correct courses appear for targeted queries

Prerequisites:
    pip install playwright requests
    playwright install chromium
    
Usage:
    python benchmark.py
"""

import json
import random
import time
import subprocess
import os
import sys
from dataclasses import dataclass
from typing import List, Dict, Any
from pathlib import Path

# Try to import playwright, give helpful error if missing
try:
    from playwright.sync_api import sync_playwright, Page
except ImportError:
    print("Please install playwright: pip install playwright && playwright install chromium")
    sys.exit(1)


@dataclass
class SearchResult:
    query: str
    num_results: int
    time_ms: float
    expected_ids: List[str]
    found_ids: List[str]
    
    @property
    def precision(self) -> float:
        """Fraction of found results that were expected"""
        if not self.found_ids:
            return 0.0
        return len(set(self.found_ids) & set(self.expected_ids)) / len(self.found_ids)
    
    @property
    def recall(self) -> float:
        """Fraction of expected results that were found"""
        if not self.expected_ids:
            return 1.0
        return len(set(self.found_ids) & set(self.expected_ids)) / len(self.expected_ids)


class SearchBenchmark:
    def __init__(self, courses_path: str = "../src/documents/courses.json"):
        with open(courses_path, "r") as f:
            self.courses = json.load(f)
        self.server_process = None
        self.port = 8080
        
    def _start_server(self):
        """Start a simple HTTP server for testing"""
        # Server needs to run from engine/ directory where index.html is
        engine_dir = Path(__file__).parent.parent
        self.server_process = subprocess.Popen(
            ["python", "-m", "http.server", str(self.port)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            cwd=engine_dir
        )
        time.sleep(1)  # Wait for server to start
        
    def _stop_server(self):
        """Stop the HTTP server"""
        if self.server_process:
            self.server_process.terminate()
            self.server_process.wait()
            
    def generate_test_queries(self) -> List[Dict[str, Any]]:
        """Generate test queries from actual course data"""
        test_cases = []
        
        # Sample random courses for testing
        course_ids = list(self.courses.keys())
        sample_courses = random.sample(course_ids, min(20, len(course_ids)))
        
        for doc_id in sample_courses:
            course = self.courses[doc_id]
            
            # Test 1: Search by exact course ID
            if course.get("courseID"):
                test_cases.append({
                    "query": course["courseID"],
                    "expected_ids": [course["courseID"]],
                    "type": "exact_id"
                })
            
            # Test 2: Search by partial course name
            if course.get("name"):
                words = course["name"].split()
                if len(words) >= 2:
                    query = " ".join(words[:2])
                    test_cases.append({
                        "query": query,
                        "expected_ids": [course.get("courseID", "")],
                        "type": "partial_name"
                    })
            
            # Test 3: Search by department
            if course.get("department"):
                test_cases.append({
                    "query": course["department"],
                    "expected_ids": [],  # Multiple results expected
                    "type": "department"
                })
                
        # Add some synthetic stress tests
        test_cases.extend([
            {"query": "programming computer science", "expected_ids": [], "type": "multi_word"},
            {"query": "introduction", "expected_ids": [], "type": "common_word"},
            {"query": "a", "expected_ids": [], "type": "single_char"},
            {"query": "architecture design studio", "expected_ids": [], "type": "long_query"},
            {"query": "xyz123nonexistent", "expected_ids": [], "type": "no_match"},
        ])
        
        return test_cases
    
    def run_search(self, page: Page, query: str) -> tuple[int, float, List[str]]:
        """Run a single search and return (num_results, time_ms, course_ids)"""
        # Clear input and type query
        page.fill("#query", "")
        page.fill("#query", query)
        
        # Wait a bit for debounce
        time.sleep(0.15)
        
        # Get timing from console or measure ourselves
        start = time.perf_counter()
        page.wait_for_function("document.querySelectorAll('.result-card').length > 0 || document.querySelector('.no-results') || document.querySelector('.search-time')", timeout=5000)
        elapsed_ms = (time.perf_counter() - start) * 1000
        
        # Try to get the actual search time from the UI
        search_time_el = page.query_selector(".search-time")
        if search_time_el:
            search_time_text = search_time_el.inner_text()
            try:
                # Extract "Found X results in Yms"
                import re
                match = re.search(r"in ([\d.]+)ms", search_time_text)
                if match:
                    elapsed_ms = float(match.group(1))
            except:
                pass
        
        # Count results
        results = page.query_selector_all(".result-card")
        num_results = len(results)
        
        # Extract course IDs from results
        course_ids = []
        for result in results:
            button = result.query_selector(".result-button")
            if button:
                course_ids.append(button.inner_text().strip())
        
        return num_results, elapsed_ms, course_ids
    
    def run_benchmarks(self) -> List[SearchResult]:
        """Run all benchmarks and return results"""
        results = []
        test_queries = self.generate_test_queries()
        
        print(f"Running {len(test_queries)} benchmark queries...")
        
        self._start_server()
        
        try:
            with sync_playwright() as p:
                browser = p.chromium.launch(headless=True)
                page = browser.new_page()
                
                # Navigate and wait for engine to initialize
                page.goto(f"http://localhost:{self.port}/index.html")
                page.wait_for_selector("#query", timeout=30000)
                print("Engine initialized, starting benchmarks...")
                
                for i, test in enumerate(test_queries):
                    try:
                        num_results, time_ms, found_ids = self.run_search(page, test["query"])
                        
                        result = SearchResult(
                            query=test["query"],
                            num_results=num_results,
                            time_ms=time_ms,
                            expected_ids=test["expected_ids"],
                            found_ids=found_ids
                        )
                        results.append(result)
                        
                        if (i + 1) % 10 == 0:
                            print(f"  Completed {i + 1}/{len(test_queries)} queries")
                            
                    except Exception as e:
                        print(f"  Error on query '{test['query']}': {e}")
                
                browser.close()
                
        finally:
            self._stop_server()
        
        return results
    
    def generate_report(self, results: List[SearchResult]) -> str:
        """Generate a markdown report of benchmark results"""
        lines = []
        lines.append("# Search Engine Benchmark Report\n")
        lines.append(f"**Date:** {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        lines.append(f"**Total Courses Indexed:** {len(self.courses)}\n")
        lines.append(f"**Queries Tested:** {len(results)}\n")
        
        # Speed statistics
        times = [r.time_ms for r in results]
        lines.append("\n## Speed Statistics\n")
        lines.append(f"| Metric | Value |")
        lines.append(f"|--------|-------|")
        lines.append(f"| Min | {min(times):.2f}ms |")
        lines.append(f"| Max | {max(times):.2f}ms |")
        lines.append(f"| Mean | {sum(times)/len(times):.2f}ms |")
        lines.append(f"| Median | {sorted(times)[len(times)//2]:.2f}ms |")
        
        # Accuracy statistics
        accuracy_results = [r for r in results if r.expected_ids]
        if accuracy_results:
            avg_recall = sum(r.recall for r in accuracy_results) / len(accuracy_results)
            lines.append(f"\n## Accuracy Statistics\n")
            lines.append(f"| Metric | Value |")
            lines.append(f"|--------|-------|")
            lines.append(f"| Average Recall | {avg_recall*100:.1f}% |")
            lines.append(f"| Queries with expected results | {len(accuracy_results)} |")
        
        # Detailed results
        lines.append("\n## Sample Results\n")
        lines.append("| Query | Results | Time (ms) | Recall |")
        lines.append("|-------|---------|-----------|--------|")
        for r in results[:25]:  # Show first 25
            recall_str = f"{r.recall*100:.0f}%" if r.expected_ids else "N/A"
            query_short = r.query[:30] + "..." if len(r.query) > 30 else r.query
            lines.append(f"| {query_short} | {r.num_results} | {r.time_ms:.2f} | {recall_str} |")
        
        return "\n".join(lines)


def main():
    print("=" * 60)
    print("WASM Search Engine Benchmark Suite")
    print("=" * 60)
    
    # Check if we're in the right directory
    if not os.path.exists("../src/documents/courses.json"):
        print("Error: Run this script from the benchmarking/ directory")
        sys.exit(1)
    
    benchmark = SearchBenchmark()
    
    print(f"\nLoaded {len(benchmark.courses)} courses from courses.json")
    
    results = benchmark.run_benchmarks()
    
    if results:
        report = benchmark.generate_report(results)
        
        # Save report
        report_path = "benchmark_report.md"
        with open(report_path, "w") as f:
            f.write(report)
        
        print(f"\n{'=' * 60}")
        print("BENCHMARK COMPLETE")
        print(f"{'=' * 60}")
        print(report)
        print(f"\nReport saved to: {report_path}")
    else:
        print("No results collected - check for errors above")


if __name__ == "__main__":
    main()
