from __future__ import annotations

import unittest

from scripts.validate_repository import (
    ROOT,
    SELF_HOSTED_PULL_REQUEST_GUARD,
    validate_self_hosted_pull_request_boundary,
)


class RepositoryWorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        self.workflow = (
            ROOT / ".github/workflows/repository-checks.yml"
        ).read_text(encoding="utf-8")

    def test_current_self_hosted_jobs_reject_fork_origin_code(self) -> None:
        self.assertEqual(
            validate_self_hosted_pull_request_boundary(self.workflow), []
        )

    def test_missing_same_repository_guard_fails(self) -> None:
        unsafe = self.workflow.replace(
            SELF_HOSTED_PULL_REQUEST_GUARD,
            "if: ${{ always() }}",
            1,
        )
        self.assertTrue(
            any(
                "same-repository head guard" in error
                for error in validate_self_hosted_pull_request_boundary(unsafe)
            )
        )

    def test_pull_request_target_fails(self) -> None:
        unsafe = self.workflow.replace(
            "  pull_request:", "  pull_request_target:", 1
        )
        errors = validate_self_hosted_pull_request_boundary(unsafe)
        self.assertTrue(any("pull_request_target" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
