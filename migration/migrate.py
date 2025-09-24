import psycopg2
import csv
from typing import List, Tuple
import re
from dataclasses import dataclass


@dataclass
class Meal:
    date: str
    time: str
    name: str
    people: List[str]
    meal_type: str
    comment: str
    location: str


@dataclass
class Day:
    date: str
    breakfast: List[Meal]
    lunch: List[Meal]
    dinner: List[Meal]
    special: str
    exercise: str
    activity: str
    entertainment: str
    housekeeping: str


def re_extract(pattern: str, text: str) -> Tuple[List[str], str]:
    matches = re.findall(pattern, text)
    for match in matches:
        text = text.replace(match, "").strip()
    return matches, text


def process_meal(s: str, date: str, time: str) -> List[Meal]:
    # re.split(r'[,，+]', i)
    meal_type = None
    m, s = re_extract(r"(bento|leftover)", s)
    if m:
        meal_type = "leftover" if "leftover" in m else "takeout"
    location = None
    m, s = re_extract(r"@.*", s)
    if m:
        location = m[0][1:].strip()
    comment = None
    m, s = re_extract(r"\(.*\)", s)
    if m:
        comment = m[0][1:-1].strip()
    m, s = re_extract(r"（.*）", s)
    if m:
        comment = m[0][1:-1].strip()
    people = []
    m, s = re_extract(r"w\/.*", s)
    if m:
        people = [p.strip() for p in re.split(r"[,，+]", m[0][2:].strip())]
    return [
        Meal(
            date=date,
            time=time,
            name=n.strip(),
            people=people,
            meal_type=meal_type,
            comment=comment,
            location=location,
        )
        for n in re.split(r"[,，+]", s)
    ]


def process_meal_time(meal_s: List[str], date: str, time: str) -> List[List[Meal]]:
    return [process_meal(s, date, time) for s in meal_s if s]


def parse(fname: str) -> List[Day]:
    days = []
    with open(fname, "r") as f:
        csv_reader = csv.reader(f.readlines())
    for row in csv_reader:
        if len(row) < 10:
            continue
        day = Day(
            date=row[1],
            breakfast=process_meal_time(row[2:4], row[1], "breakfast"),
            lunch=process_meal_time(row[4:6], row[1], "lunch"),
            dinner=process_meal_time(row[6:8], row[1], "dinner"),
            special=row[8],
            exercise=row[9],
            activity=row[10],
            entertainment=row[11],
            housekeeping=row[12],
        )
        days.append(day)
    return days


def get_who(food_source: str, idx: int, total: int) -> List[str]:
    if food_source == "recipe" or total == 1:
        return ["ww", "xx"]
    return ["ww"] if idx == 1 else ["xx"]


def extract_meal(
    keyword: str,
    days: List[Day],
    food_source: str = None,
    exact: bool = True,
    dry_run: bool = False,
) -> List[Meal]:
    def match(meal: Meal) -> bool:
        if exact:
            return keyword.lower() == meal.name.lower()
        return keyword.lower() in meal.name.lower()

    rtn = []
    for day in days:
        for time in ["breakfast", "lunch", "dinner"]:
            meals = getattr(day, time)
            extracted_idx = []
            for i, cell in enumerate(meals):
                who = get_who(food_source, i, len(meals))
                for j, meal in enumerate(cell):
                    if match(meal):
                        meal.people += who
                        rtn.append(meal)
                        extracted_idx.append((i, j))
            if not dry_run and extracted_idx:
                setattr(
                    day,
                    time,
                    [
                        [
                            meal
                            for j, meal in enumerate(cell)
                            if (i, j) not in extracted_idx
                        ]
                        for i, cell in enumerate(meals)
                    ],
                )
    print(f"{'Found' if dry_run else 'Extracted'} {len(rtn)} records for '{keyword}'")
    return rtn


def merge_meal(meals: List[Meal]) -> List[Meal]:
    if not meals:
        return []
    rtn = [meals[0]]
    for m in meals[1:]:
        if (
            m.date == rtn[-1].date
            and m.name == rtn[-1].name
            and m.time == rtn[-1].time
            and m.meal_type == rtn[-1].meal_type
            and set(m.people + rtn[-1].people) == set(["xx", "ww"])
        ):
            rtn[-1].people = ["xx", "ww"]
        else:
            rtn.append(m)
    if len(rtn) < len(meals):
        print(f"Merged {len(meals) - len(rtn)} records")
    return rtn


def get_food_id(
    conn: psycopg2.extensions.connection, food_name: str, food_source: str
) -> int:
    cur = conn.cursor()
    cur.execute(f"SELECT id FROM {food_source} WHERE name = %s", (food_name,))
    row = cur.fetchone()
    conn.commit()
    cur.close()
    return row[0] if row else -1


def get_people_id(conn: psycopg2.extensions.connection, people_names: List[str]) -> int:
    cur = conn.cursor()
    cur.execute("SELECT id FROM people WHERE name = ANY(%s)", (people_names,))
    rows = cur.fetchall()
    conn.commit()
    cur.close()
    return [r[0] for r in rows]


def direct_insert_meal(
    conn: psycopg2.extensions.connection,
    meals: List[Meal],
    food_name: str,
    food_source: str,
    meal_type_override: str = None,
    comment_override: str = None,
) -> bool:
    fid = get_food_id(conn, food_name, food_source)
    if fid == -1:
        print(f"Food item '{food_name}' not found in '{food_source}' table.")
        return False
    for meal in meals:
        people_ids = get_people_id(conn, meal.people)
        if not people_ids:
            print(f"People '{meal.people}' not found in 'people' table.")
            return False
        # insert into meal table and get auto-generated id
        cur = conn.cursor()
        comment = meal.comment if meal.comment else comment_override
        cur.execute(
            "INSERT INTO meal (date, time, notes) VALUES (%s, %s, %s) RETURNING id",
            (meal.date, meal.time, comment),
        )
        meal_id = cur.fetchone()[0]
        conn.commit()
        cur.close()
        # insert into meal_<food_source> table
        cur = conn.cursor()
        mtm = {"restaurant": "dine-in", "product": "manufactured", "recipe": "cooked"}
        meal_type = (
            meal.meal_type
            if meal.meal_type
            else (meal_type_override if meal_type_override else mtm[food_source])
        )
        cur.execute(
            f"INSERT INTO meal_{food_source} (meal, {food_source}, type) VALUES (%s, %s, %s)",
            (meal_id, fid, meal_type),
        )
        conn.commit()
        cur.close()
        # insert into meal_people table
        for pid in people_ids:
            cur = conn.cursor()
            cur.execute(
                "INSERT INTO meal_people (meal, people) VALUES (%s, %s)", (meal_id, pid)
            )
            conn.commit()
            cur.close()
    print(f"Inserted {len(meals)} records for '{food_name}' of '{food_source}'")
    return True


def clear_names(names: List[str], days: List[Day], exact: bool = True):
    for name in names:
        extract_meal(name, days, exact=exact)


def insert_one_food(
    conn: psycopg2.extensions.connection,
    days: List[Day],
    food_name: str,
    food_source: str,
    replace_name: str = None,
    exact: bool = True,
    meal_type_override: str = None,
    comment_override: str = None,
):
    if not replace_name:
        replace_name = food_name
    meals = merge_meal(
        extract_meal(
            food_name, days, food_source=food_source, exact=exact, dry_run=True
        )
    )
    if not meals:
        return
    if direct_insert_meal(
        conn,
        meals,
        replace_name,
        food_source,
        meal_type_override=meal_type_override,
        comment_override=comment_override,
    ):
        extract_meal(food_name, days, food_source=food_source, exact=exact)


conn = psycopg2.connect(dbname="xnote", user="postgres", host="localhost", port="5432")
data = parse("old_data.csv")
clear_names(
    [
        "milk",
        "十九金",
        "巧克力派",
        "巧克力面包",
        "Google 食堂",
        "土豆烧排骨",
        "醋溜白菜",
        "蛋炒饭",
        "小锅米线",
        "酱肘子",
        "成都滋味",
        "红烧金鲳鱼",
        "炝炒油麦菜",
        "红烧牛肉面",
        "西红柿鸡蛋面",
        "煮饺子",
        "蛋黄派",
        "ritz",
        "重庆小面",
        "煮速冻饺子",
        "番茄土豆炖牛腩",
        "红烧平鱼",
        "炒油菜",
        "小葱拌豆腐",
        "牛丼",
        "猪脚饭",
    ],
    data,
)
clear_names(
    ["盐焗鸡", "YGF", "McDonald", "西红柿鸡蛋面", "croissant", "方便面", "阳春面"],
    data,
    exact=False,
)
