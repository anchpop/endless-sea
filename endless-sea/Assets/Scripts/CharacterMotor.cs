#nullable enable
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class CharacterMotor : MonoBehaviour
{
#nullable disable
    // Place public fields to be set in the inspector here
#nullable restore

    // Start is called before the first frame update
    void Start()
    {

    }

    // Update is called once per frame
    void Update()
    {
        // move on x and z axis as per input
        var x = Input.GetAxis("Horizontal");
        var z = Input.GetAxis("Vertical");
        transform.position += new Vector3(x, 0, z).normalized * Time.deltaTime * 5;
    }
}

#nullable restore
