# Disko

Disko is a disk space visualization tool designed for Linux systems. It provides an intuitive graphical representation of your directory structure through pie charts and tree views, offering real-time insights into the composition of your storage.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)

## Features

### 1. Pie Chart
- Dynamic visualization of directory structure.
- Real-time updates on rescanning directories.
- Support for both directories and files.
- Two views: default "Pie" view and "Special Pie" view.

### 2. Tree-view Representation
- Expandable tree-view for directory representation.
- Dynamic updates on node expansion.
- Accurate reflection of directory structure in real-time.

### 3. Rescan Capability
- Dynamic rescanning for exploring new directories.
- Manual input or search window for directory selection.
- Real-time updates in tree and chart views.

### 4. Menus
#### 4.1 "Configurations" Menu
- View and edit options for configurations.
- Configure based on min and max item sizes, include/exclude files, specify max depth, and use regex patterns.

#### 4.2 "Group" Menu
- Group by size or extension.
- Pie charts for size distribution or file extension breakdown.

#### 4.3 "Get Files Sorted" Menu
- Sort files by name or size.
- Display of sorted files in ascending or descending order.

#### 4.4 "Get" Menu
- Cleanup recommendations, directory size history, and report generation options.
- Practical tools for real-time cleanup and historical analysis.

## Installation

How to use Disko?
- Dependencies and required libraries:
  Prior to running the Disko application, ensure the installation of the following
  * dependencies:
    1. rustup
    2. Cargo
  * Installation Steps:
    1. Clone the Disko repository by executing the following command in the
    terminal:
      git clone https://github.com/youssifabuzied/Disko
    2. Navigate to the Disko directory using the terminal:
      cd Disko
    3. Run the application using Cargo:
      cargo run


