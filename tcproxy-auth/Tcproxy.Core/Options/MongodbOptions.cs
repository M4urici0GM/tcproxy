using System.ComponentModel.DataAnnotations;

namespace Tcproxy.Core.Options;

public class MongodbOptions
{
    [Required]
    public string ConnectionString { get; set; } = string.Empty;

    [Required]
    public string DatabaseName { get; set; } = string.Empty;
}