import psycopg2
import csv
from typing import List, Tuple
import re
from dataclasses import dataclass

@dataclass
class Day:
    date: str
    breakfast: List[str]
    lunch: List[str]
    dinner: List[str]
    special: str
    exercise: str
    activity: str
    entertainment: str
    housekeeping: str

def parse(fname: str) -> List[Day]:
    days = []
    with open(fname, 'r') as f:
        csv_reader = csv.reader(f.readlines())
    for row in csv_reader:
        if len(row) < 10:
            continue
        day = Day(
            date=row[1],
            breakfast=[row[2]] + ([row[3]] if row[3] else []),
            lunch=[row[4]] + ([row[5]] if row[5] else []),
            dinner=[row[6]] + ([row[7]] if row[7] else []),
            special=row[8],
            exercise=row[9],
            activity=row[10],
            entertainment=row[11],
            housekeeping=row[12]
        )
        days.append(day)
    return days

def process(days: List[Day]):
    def flatten(meals: List[str]) -> List[str]:
        rtn = []
        for i in meals:
            rtn.append([j.strip() for j in re.split(r'[,，+]', i) if j.strip()])
        return rtn
    for i in range(len(days)):
        days[i].breakfast = flatten(days[i].breakfast)
        days[i].lunch = flatten(days[i].lunch)
        days[i].dinner = flatten(days[i].dinner)

def extract(keyword: str, days: List[Day], food_source: str = None, exact: bool = True, dry_run: bool = False) -> List[Tuple[str, str, str]]:
    def match(text: str) -> bool:
        if exact:
            return text == keyword
        return keyword in text
    rtn = []
    for day in days:
        for key in dir(day):
            if key.startswith('_') or key == 'date':
                continue
            value = getattr(day, key)
            if isinstance(value, list):
                # print(value)
                extracted_idx = []
                for i, cell in enumerate(value):
                    who = 'both' if food_source == 'recipe' else ('ww' if i == 1 else ('both' if len(value) == 1 else 'xx'))
                    for j, item in enumerate(cell):
                        if match(item):
                            rtn.append((day.date, key, who, ))
                            extracted_idx.append((i, j))
                if not dry_run and extracted_idx:
                    setattr(day, key, [[v for j, v in enumerate(cell) if (i, j) not in extracted_idx] for i, cell in enumerate(value)])
            else: # str
                if match(value):
                    rtn.append((day.date, key))
                    if not dry_run:
                        setattr(day, key, '')
    return rtn

def merge_meal(entries: List[Tuple[str, str, str]]) -> List[Tuple[str, str, str]]:
    rtn = [list(entries[0])]
    for d, k, p in entries[1:]:
        if d == rtn[-1][0] and k == rtn[-1][1]:
            if set([rtn[-1][2], p]) == set(['xx', 'ww']):
                rtn[-1][2] = 'both'
        else:
            rtn.append([d, k, p])
    return [tuple(i) for i in rtn]

def get_food_id(conn: psycopg2.extensions.connection, food_name: str, food_source: str) -> int:
    cur = conn.cursor()
    cur.execute(f"SELECT id FROM {food_source} WHERE name = %s", (food_name,))
    row = cur.fetchone()
    conn.commit()
    cur.close()
    return row[0] if row else -1

def get_people_id(conn: psycopg2.extensions.connection, people_names: List[str] = ['both']) -> int:
    if people_names == ['both']:
        people_names = ['xx', 'ww']
    cur = conn.cursor()
    cur.execute("SELECT id FROM people WHERE name = ANY(%s)", (people_names,))
    rows = cur.fetchall()
    conn.commit()
    cur.close()
    return [r[0] for r in rows]

def direct_insert_meal(conn: psycopg2.extensions.connection, entries: List[Tuple[str, str, str]], food_name: str, food_source: str):
    # conn = psycopg2.connect(dbname="xnote_dev", user="postgres", host="localhost", port="5432")
    fid = get_food_id(conn, food_name, food_source)
    if fid == -1:
        print(f"Food item '{food_name}' not found in '{food_source}' table.")
        return
    for date, time, people in entries:
        people_ids = get_people_id(conn, [people])
        # insert into meal table and get auto-generated id
        cur = conn.cursor()
        cur.execute("INSERT INTO meal (date, time) VALUES (%s, %s) RETURNING id", (date, time))
        meal_id = cur.fetchone()[0]
        conn.commit()
        cur.close()
        # insert into meal_<food_source> table
        cur = conn.cursor()
        mtm = {
            'restaurant': 'dine-in',
            'product': 'manufactured',
            'recipe': 'cooked'
        }
        cur.execute(f"INSERT INTO meal_{food_source} (meal, {food_source}, type) VALUES (%s, %s, %s)", (meal_id, fid, mtm[food_source]))
        conn.commit()
        cur.close()
        # insert into meal_people table
        for pid in people_ids:
            cur = conn.cursor()
            cur.execute("INSERT INTO meal_people (meal, people) VALUES (%s, %s)", (meal_id, pid))
            conn.commit()
            cur.close()

def clear_names(names: List[str], days: List[Day], exact: bool = True):
    for name in names:
        extract(name, days, exact=exact, dry_run=False)

def insert_one_food(conn: psycopg2.extensions.connection, days: List[Day], food_name: str, food_source: str, replace_name: str = None, exact: bool = True):
    if not replace_name:
        replace_name = food_name
    direct_insert_meal(conn, merge_meal(extract(food_name, days, food_source=food_source, exact=exact)), replace_name, food_source)


conn = psycopg2.connect(dbname="xnote", user="postgres", host="localhost", port="5432")
data = parse('old_data.csv')
process(data)
clear_names(['milk', '十九金', '巧克力派', '巧克力面包', 'Google 食堂', '土豆烧排骨', '醋溜白菜', '蛋炒饭', '小锅米线'], data)
clear_names(['盐焗鸡'], data, exact=False)
