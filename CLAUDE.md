# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **xnote**, a Rust-based REST API service for tracking daily events (meals, activities, drinks) built with Actix Web and PostgreSQL. The service allows users to log what they ate, what activities they did, and who they did them with.

## Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Run the server (requires DATABASE_URL in .env)
cargo run

# Server runs on http://0.0.0.0:8080
```

### Environment Setup
```bash
# Copy environment template and configure
cp .env.example .env
# Edit .env with your PostgreSQL connection string
```

### Database Setup
```bash
# Run the database schema (PostgreSQL required)
psql -d your_database -f init.sql
```

## Architecture Overview

### Core Concept: Daily Event Tracking
The application tracks three main types of daily events:
- **Meals**: What you ate (from recipes, products, or restaurants) and with whom
- **Activities**: What you did, where, with whom (sports, work, entertainment, etc.)
- **Drinks**: What you drank and with whom

### Database Design Principles
- **Mutually Exclusive Food Sources**: Each meal associates with exactly ONE of: recipe, product, or restaurant
- **Associative Tables**: Junction tables link events to people and categorize meal types
- **Enum Tables**: Lookup tables for consistent categorization (meal_time, activity_type, etc.)

### API Architecture

#### Endpoint Patterns
- **Simple CRUD**: `/api/v1/{resource}` and `/api/v1/{resource}/{id}`
- **Rich Details**: `/api/v1/{resource}/{id}/details` - aggregated responses with full context
- **Enum-like Resources**: Only GET, POST, DELETE (no individual GET by ID)

#### Response Models
The service implements **dual response patterns**:

1. **Simple Models**: Basic entity data (`Meal`, `Event`, `Restaurant`, etc.)
2. **Detail Models**: Rich aggregated responses with full context
   - `MealDetail`: Includes food source (recipe/product/restaurant) + people
   - `EventDetail`: Includes full activity details + people  
   - `DrinkDetail`: Includes people

### Code Organization

```
src/
├── main.rs              # Actix server setup and routing
├── models/              # Data structures
│   ├── {entity}.rs      # Simple CRUD models
│   └── detail.rs        # Rich aggregated response models
├── handlers/            # HTTP endpoint implementations
│   └── {entity}.rs      # REST endpoints per entity
└── config/              # Database configuration
```

### Key Technical Patterns

#### SQLx Nullable Field Handling
For LEFT JOIN queries that may return NULL values, use SQLx nullable annotations:
```rust
// In sqlx::query! macro
r.id as "recipe_id?", r.name as "recipe_name?"
```

#### Food Source Discrimination  
Meals use an enum to represent mutually exclusive food sources:
```rust
pub enum MealFoodSource {
    Recipe { recipe: Recipe, meal_type: String },
    Product { product: Product, meal_type: String },
    Restaurant { restaurant: Restaurant, meal_type: String },
}
```

#### SQLx Rename Annotations
For database columns that conflict with Rust keywords:
```rust
#[serde(rename = "type")]      // JSON serialization
#[sqlx(rename = "type")]       // Database column mapping  
pub activity_type: String,
```

### Database Schema Key Points

- **Junction Tables**: `meal_recipe`, `meal_product`, `meal_restaurant`, `meal_people`, `event_people`, `drink_people`
- **Enum Tables**: `meal_time`, `meal_type`, `activity_type`, `food_type`, `drink_option`, `location`
- **Time Field**: Quoted as `"time"` in SQL queries (PostgreSQL reserved keyword)
- **Cascading Deletes**: Junction tables cascade delete when parent entities are removed

### API Endpoint Summary

#### Basic CRUD
- Meals, Events, People, Restaurants, Activities, Products, Recipes, Drinks

#### Enum-like Resources (GET/POST/DELETE only)
- Locations, Drink-options

#### Rich Aggregated Endpoints
- `GET /api/v1/meals/{id}/details` - Meal with food source + people
- `GET /api/v1/events/{id}/details` - Event with activity details + people
- `GET /api/v1/drinks/{id}/details` - Drink with people

#### Health Check
- `GET /health` - Service health status