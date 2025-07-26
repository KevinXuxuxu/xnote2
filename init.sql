-- table definitions
CREATE TABLE IF NOT EXISTS location (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS food_type (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS restaurant (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    type TEXT NOT NULL,
    price REAL,
    FOREIGN KEY (location) REFERENCES location(name),
    FOREIGN KEY (type) REFERENCES food_type(name)
);

CREATE TABLE IF NOT EXISTS product (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recipe (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,
    procedure TEXT NOT NULL,
    cautions TEXT
);

CREATE TABLE IF NOT EXISTS meal_time (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS meal_type (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS people (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS meal (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    "time" TEXT NOT NULL,
    notes TEXT,
    FOREIGN KEY ("time") REFERENCES meal_time(name)
);

CREATE TABLE IF NOT EXISTS meal_recipe (
    meal INTEGER NOT NULL,
    recipe INTEGER NOT NULL,
    type TEXT NOT NULL,
    PRIMARY KEY (meal, recipe),
    FOREIGN KEY (meal) REFERENCES meal(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe) REFERENCES recipe(id),
    FOREIGN KEY (type) REFERENCES meal_type(name)
);

CREATE TABLE IF NOT EXISTS meal_product (
    meal INTEGER NOT NULL,
    product INTEGER NOT NULL,
    type TEXT NOT NULL,
    PRIMARY KEY (meal, product),
    FOREIGN KEY (meal) REFERENCES meal(id) ON DELETE CASCADE,
    FOREIGN KEY (product) REFERENCES product(id),
    FOREIGN KEY (type) REFERENCES meal_type(name)
);

CREATE TABLE IF NOT EXISTS meal_restaurant (
    meal INTEGER NOT NULL,
    restaurant INTEGER NOT NULL,
    type TEXT NOT NULL,
    PRIMARY KEY (meal, restaurant),
    FOREIGN KEY (meal) REFERENCES meal(id) ON DELETE CASCADE,
    FOREIGN KEY (restaurant) REFERENCES restaurant(id),
    FOREIGN KEY (type) REFERENCES meal_type(name)
);

CREATE TABLE IF NOT EXISTS meal_people (
    meal INTEGER NOT NULL,
    people INTEGER NOT NULL,
    PRIMARY KEY (meal, people),
    FOREIGN KEY (meal) REFERENCES meal(id) ON DELETE CASCADE,
    FOREIGN KEY (people) REFERENCES people(id)
);

CREATE TABLE IF NOT EXISTS activity_type (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS activity (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    FOREIGN KEY (type) REFERENCES activity_type(name)
);

CREATE TABLE IF NOT EXISTS event (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    activity INTEGER NOT NULL,
    measure TEXT,
    location TEXT,
    notes TEXT,
    FOREIGN KEY (activity) REFERENCES activity(id)
);

CREATE TABLE IF NOT EXISTS event_people (
    event INTEGER NOT NULL,
    people INTEGER NOT NULL,
    PRIMARY KEY (event, people),
    FOREIGN KEY (event) REFERENCES event(id) ON DELETE CASCADE,
    FOREIGN KEY (people) REFERENCES people(id)
);

CREATE TABLE IF NOT EXISTS drink_option (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS drink (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    date DATE NOT NULL,
    FOREIGN KEY (name) REFERENCES drink_option(name)
);

CREATE TABLE IF NOT EXISTS drink_people (
    drink INTEGER NOT NULL,
    people INTEGER NOT NULL,
    PRIMARY KEY (drink, people),
    FOREIGN KEY (drink) REFERENCES drink(id) ON DELETE CASCADE,
    FOREIGN KEY (people) REFERENCES people(id)
);

-- enum table initialization
INSERT INTO location (name) VALUES ('SLU');
INSERT INTO location (name) VALUES ('Seattle Downtown');
INSERT INTO location (name) VALUES ('Northgate');
INSERT INTO location (name) VALUES ('Capitol Hill');
INSERT INTO location (name) VALUES ('Wallingford');
INSERT INTO location (name) VALUES ('Bellevue');
INSERT INTO location (name) VALUES ('Ballard');
INSERT INTO location (name) VALUES ('U District');
INSERT INTO location (name) VALUES ('Lynwood');
INSERT INTO location (name) VALUES ('Fremont');
INSERT INTO location (name) VALUES ('Chinatown');
INSERT INTO location (name) VALUES ('Kirkland');
INSERT INTO location (name) VALUES ('NYC');
INSERT INTO location (name) VALUES ('Portland');
INSERT INTO location (name) VALUES ('Olympia');
INSERT INTO location (name) VALUES ('Bay Area');
INSERT INTO location (name) VALUES ('Columbus');
INSERT INTO location (name) VALUES ('Hawaii');
INSERT INTO location (name) VALUES ('');

INSERT INTO food_type (name) VALUES ('mexican');
INSERT INTO food_type (name) VALUES ('japanese');
INSERT INTO food_type (name) VALUES ('chinese');
INSERT INTO food_type (name) VALUES ('pizza');
INSERT INTO food_type (name) VALUES ('fast food');
INSERT INTO food_type (name) VALUES ('cafeteria');
INSERT INTO food_type (name) VALUES ('salad');
INSERT INTO food_type (name) VALUES ('korean');
INSERT INTO food_type (name) VALUES ('Turkish');
INSERT INTO food_type (name) VALUES ('seafood');
INSERT INTO food_type (name) VALUES ('vietnamese');
INSERT INTO food_type (name) VALUES ('Italian');
INSERT INTO food_type (name) VALUES ('Indian');
INSERT INTO food_type (name) VALUES ('Thai');
INSERT INTO food_type (name) VALUES ('brunch');
INSERT INTO food_type (name) VALUES ('');

INSERT INTO meal_time (name) VALUES ('breakfast');
INSERT INTO meal_time (name) VALUES ('lunch');
INSERT INTO meal_time (name) VALUES ('dinner');

INSERT INTO meal_type (name) VALUES ('cooked');
INSERT INTO meal_type (name) VALUES ('dine-in');
INSERT INTO meal_type (name) VALUES ('takeout');
INSERT INTO meal_type (name) VALUES ('manufactured');
INSERT INTO meal_type (name) VALUES ('leftover');

INSERT INTO activity_type (name) VALUES ('chore');
INSERT INTO activity_type (name) VALUES ('vedio game');
INSERT INTO activity_type (name) VALUES ('housekeeping');
INSERT INTO activity_type (name) VALUES ('work');
INSERT INTO activity_type (name) VALUES ('sport');
INSERT INTO activity_type (name) VALUES ('side project');
INSERT INTO activity_type (name) VALUES ('study');
INSERT INTO activity_type (name) VALUES ('entertainment');
INSERT INTO activity_type (name) VALUES ('transport');