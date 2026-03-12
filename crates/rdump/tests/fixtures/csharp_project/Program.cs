using System;
using Demo.Utils;

namespace Demo.App
{
    interface IGreeter
    {
        void Greet(string name);
    }

    class Greeter : IGreeter
    {
        public void Greet(string name)
        {
            Console.WriteLine($"Hello {name}");
        }
    }

    struct Point
    {
        public int X;
        public int Y;
    }

    static class Program
    {
        static int Add(int a, int b) => a + b;

        static void Main(string[] args)
        {
            IGreeter g = new Greeter();
            g.Greet("world");
            var p = new Point { X = 1, Y = 2 };
            Console.WriteLine(Add(p.X, p.Y));
            Helper.DoNothing();
        }
    }
}
