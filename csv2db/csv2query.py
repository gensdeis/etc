import pandas as pd

# CSV 파일 경로
csv_file = 'gamedb_20240823.csv'

# 쿼리 파일 경로
query_file = 'create_descriptions.sql'

# CSV 파일 읽기
df = pd.read_csv(csv_file)

# 쿼리 문자열을 저장할 리스트
queries = []

# 테이블 및 컬럼 설명 추가
tables = df['TABLE NAME'].dropna().unique()

for table in tables:
    # 테이블에 해당하는 데이터만 필터링
    table_df = df[df['TABLE NAME'] == table]
    
    # 테이블 설명 추가
    if not table_df['TABLE DESCRIPTION'].dropna().empty:
        table_description = table_df['TABLE DESCRIPTION'].dropna().iloc[0]
        queries.append(f"""
EXEC sp_addextendedproperty 
    'MS_Description', 
    '{table_description}', 
    'user', 
    'dbo', 
    'table', 
    '{table}';
GO
""")
    
    # 컬럼 설명 추가
    for _, row in table_df.iterrows():
        column_name = row['COLUMN NAME']
        column_description = row['COLUMN DESCRIPTION']
        
        if pd.notna(column_name) and pd.notna(column_description):
            # 컬럼 이름 및 설명에 따옴표가 포함된 경우 이스케이프 처리
            column_name_escaped = column_name.replace("'", "''")
            column_description_escaped = column_description.replace("'", "''")
            queries.append(f"""
EXEC sp_addextendedproperty 
    'MS_Description', 
    '{column_description_escaped}', 
    'user', 
    'dbo', 
    'table', 
    '{table}', 
    'column', 
    '{column_name_escaped}';
GO
""")

# 쿼리 파일에 저장
with open(query_file, 'w', encoding='utf-8') as f:
    f.write('\n'.join(queries))

print(f"쿼리 파일이 '{query_file}'로 저장되었습니다.")
