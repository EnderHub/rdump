module Main where

greet :: String -> IO ()
greet name = putStrLn ("Hello " ++ name)

add :: Int -> Int -> Int
add a b = a + b

main :: IO ()
main = do
  greet "world"
  print (add 1 2)
