using System.ComponentModel.DataAnnotations;

namespace tcproxy.core.Options;

public class MongodbOptions
{
    [Required]
    public string ConnectionString { get; set; } = string.Empty;

    [Required]
    public string DatabaseName { get; set; } = string.Empty;
}