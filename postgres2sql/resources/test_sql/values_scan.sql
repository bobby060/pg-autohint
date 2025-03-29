SELECT temp1.col2, temp1.col3
FROM (
  VALUES 
    (1,2,3),
    (4,5,6)
) AS temp1(col1, col2, col3) INNER JOIN (
  VALUES
    (1,10,100),
    (4,40,400)
) AS temp2(col1, col2, col3) ON temp1.col1 = temp2.col2
WHERE temp1.col2 > 2;