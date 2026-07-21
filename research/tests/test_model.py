"""Unit tests for data model types."""

from __future__ import annotations

from kuberina.model.types import ResourceVector


def test_resource_vector_fits_within_capacity() -> None:
    """A larger vector should fit a smaller demand."""
    capacity = ResourceVector(cpu=4.0, ram=16.0, gpu=1.0)
    demand = ResourceVector(cpu=2.0, ram=8.0, gpu=1.0)
    assert capacity.fits(demand) is True


def test_resource_vector_rejects_overcapacity() -> None:
    """Demand exceeding any single dimension should fail."""
    capacity = ResourceVector(cpu=4.0, ram=16.0, gpu=0.0)
    demand = ResourceVector(cpu=2.0, ram=20.0, gpu=0.0)
    assert capacity.fits(demand) is False


def test_resource_vector_subtract() -> None:
    """Subtraction should produce correct residual."""
    a = ResourceVector(cpu=4.0, ram=16.0, gpu=2.0)
    b = ResourceVector(cpu=1.0, ram=4.0, gpu=1.0)
    result = a.subtract(b)
    assert result.cpu == 3.0
    assert result.ram == 12.0
    assert result.gpu == 1.0


def test_resource_vector_add() -> None:
    """Addition should accumulate resources."""
    a = ResourceVector(cpu=1.0, ram=4.0)
    b = ResourceVector(cpu=0.5, ram=2.0)
    result = a.add(b)
    assert result.cpu == 1.5
    assert result.ram == 6.0


def test_resource_vector_zero() -> None:
    """Zero vector should have all dimensions at 0."""
    zero = ResourceVector.zero()
    assert zero.cpu == 0.0
    assert zero.ram == 0.0
    assert zero.gpu == 0.0
