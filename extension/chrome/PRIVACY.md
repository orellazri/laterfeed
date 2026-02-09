# Laterfeed Chrome Extension — Privacy Policy

**Last updated:** February 9, 2026

## Overview

The Laterfeed Chrome extension ("the Extension") is a companion tool for the self-hosted Laterfeed application. It allows users to save web pages to their own Laterfeed server instance.

## Data Collection

The Extension collects and handles the following data:

### Authentication Information

- An **authentication token** provided by the user is stored locally in the browser using `chrome.storage.sync`. This token is used solely to authenticate requests to the user's own Laterfeed server. It is never sent to any third party.

### Website Content

- When the user clicks the Extension icon, the **URL of the currently active tab** is read and pre-filled into the save form. This URL, along with an optional user-provided title, is sent to the user's own self-hosted Laterfeed server when the user explicitly clicks "Save".

### User Configuration

- The **base URL** of the user's Laterfeed server instance is stored locally in the browser using `chrome.storage.sync`.

## Data Storage

All data is stored locally in the user's browser via `chrome.storage.sync`. If the user is signed in to Chrome, this data may sync across their Chrome instances per Chrome's built-in sync behavior.

## Data Sharing

The Extension does **not**:

- Sell or transfer user data to third parties
- Send data to any server other than the user's own self-hosted Laterfeed instance
- Track browsing history or user activity
- Collect analytics or telemetry
- Use data for advertising, creditworthiness, or lending purposes

## Third-Party Services

The Extension communicates exclusively with the user's own self-hosted Laterfeed server. No other third-party services are contacted.

## Changes to This Policy

Any updates to this privacy policy will be reflected in this document with an updated date.

## Contact

For questions or concerns about this privacy policy, please open an issue at [https://github.com/orellazri/laterfeed](https://github.com/orellazri/laterfeed).
