# Structured Info

## Purpose

This document defines the expected output of data processing.

## Input

For each information source, the system has a data collection method documented in [methods](methods/overview.md) and [providers](providers/).

Each source may have its own request format, access method, raw response format, limitations, and fallback path.

## Output modes

The same source scripts should provide two output modes:

1. Raw data

   All data that the source script received or extracted from the underlying source, without product-level structuring.

   For a CLI source, this is the captured CLI output. For an API source, this is the response received from the API. For a local-file source, this is the data that the source script read or extracted according to its collection method.

   Raw data does not have to be stable between runs and does not have to follow a common schema.

2. Structured data

   Data converted into the common product-level structure defined below.

   Structured data must be stable and machine-readable. It must follow the same field contract for every provider and source.

User-facing presentation is a separate layer. Source scripts should not define the final terminal summary format, limit bars, colors, provider headers, or fallback display text.

The field schema is documented in [structured-info-schema.md](structured-info-schema.md); field-population rules are documented in [structured-info-rules.md](structured-info-rules.md).
