using FizzWare.NBuilder;

namespace Tcproxy.Tests.Utils;

public class FakeIt
{
  public static T FakeSingle<T>() where T : class
  {
    return Builder<T>
      .CreateNew()
      .Build();
  }

  public static IEnumerable<T> FakeMultiple<T>(int size = 30) where T : class
  {
    return Builder<T>
      .CreateListOfSize(size)
      .All()
      .Build()
      .ToList();
  }
}